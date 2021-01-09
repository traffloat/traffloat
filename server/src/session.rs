use std::collections::VecDeque;
use std::fmt;

use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};

use std::time::Duration;
use std::time::Instant;

use anyhow::{Context, Result};
use futures::channel::{mpsc, oneshot};
use futures::future::FutureExt;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use tokio::net;
use tokio::sync::{Mutex, RwLock};
use tokio::time as tt;
use tungstenite::protocol::CloseFrame;
use tungstenite::Message;

use crate::util::{TimeoutExt, PING_FREQ, STD_TIMEOUT};
use common::proto::{handshake, Packet};

static NEXT_SESSION_ID: AtomicUsize = AtomicUsize::new(0);

const PING_BACKLOG_SIZE: usize = 10;

pub async fn handle(
    conn: net::TcpStream,
    addr: SocketAddr,
    update: mpsc::Sender<Update>,
) -> Result<()> {
    let mut conn = tokio_tungstenite::accept_async(conn)
        .std_timeout()
        .await??;

    // read proto cksum
    {
        let proto_cksum = match conn.next().std_timeout().await? {
            None => return Ok(()),
            Some(Err(err)) => return Err(err).context("Error receiving version handshake"),
            Some(Ok(Message::Binary(buf))) => buf,
            _ => anyhow::bail!("Unexpected handshake message type"),
        };
        if proto_cksum[..] != common::proto::VERSION[..] {
            let error = format!(
                "Incompatible protocol versions (client = {:032x?}, server = {:032x?})",
                &proto_cksum[..],
                &common::proto::VERSION[..]
            );
            return Err(anyhow::Error::msg(error));
        }

        conn.send(Message::Binary(common::proto::VERSION[..].to_vec()))
            .std_timeout()
            .await??;
    }

    let (send_sender, mut send_receiver) = mpsc::channel(16);
    let (close_sender, close_receiver) = oneshot::channel();
    let mut close_receiver = close_receiver.fuse();
    let session = Session::new(addr, send_sender, close_sender, update.clone());

    // Ok(time) => send next ping at time
    // Err(time) => sent last ping at time
    let mut next_ping: Result<Instant, Instant> = Ok(Instant::now() + PING_FREQ);
    let mut timeout = Instant::now() + STD_TIMEOUT;

    let result = async {
        loop {
            futures::select! {
                message = conn.next().fuse() => {
                    let message = match message {
                        Some(message) => message,
                        None => return Ok(()),
                    }.context("Error receiving message")?;
                    match message {
                        Message::Binary(buf) => {
                            session.recv(&buf[..]).await.context("Packet decode error")?;
                        }
                        Message::Ping(buf) => {
                            conn.send(Message::Pong(buf)).await.context("Error replying to ping packet")?;
                        }
                        Message::Pong(_buf) => {
                            timeout = Instant::now() + STD_TIMEOUT;
                            let ping = match next_ping {
                                Ok(_) => anyhow::bail!("Multiple pong packets received"),
                                Err(time) => time,
                            };
                            session.add_ping(ping.elapsed()).await;
                            next_ping = Ok(Instant::now() + PING_FREQ);
                        }
                        Message::Close(frame) => {
                            if let Some(frame) = frame {
                                log::debug!("Connection {} closed: {:?} ({})", addr, frame.code, frame.reason.as_ref());
                            } else {
                                log::debug!("Connection {} closed", addr);
                            }
                            return Ok(());
                        }
                        _ => anyhow::bail!("Received unexpected message type"),
                    }
                }
                packet = send_receiver.next() => {
                    let packet = match packet {
                        Some(packet) => packet,
                        None => return Ok(()),
                    };
                    conn.send(Message::Binary(packet)).await.context("Error writing message")?;
                }
                frame = close_receiver => {
                    let frame = frame.expect("Session dropped");
                    conn.send(Message::Close(Some(frame))).await.context("Error closing socket")?;
                    return Ok(());
                }
                _ = tt::sleep_until(match next_ping {
                    Ok(ping) => ping,
                    Err(_) => Instant::now() + STD_TIMEOUT * 2 // never resolves
                }.into()).fuse() => {
                    log::debug!("Sending ping");
                    next_ping = Err(Instant::now());
                    conn.send(Message::Ping(Vec::from(b"ping".as_ref()))).await.context("Error sending ping")?;
                }
                _ = tt::sleep_until(timeout.into()).fuse() => anyhow::bail!("Ping timeout"),
            }
        }
    }.await;
    session.set_closed().await;
    result
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SessionId(pub usize);

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(getset::CopyGetters)]
pub struct Session {
    #[get_copy = "pub"]
    id: SessionId,
    #[get_copy = "pub"]
    addr: SocketAddr,
    send: mpsc::Sender<Vec<u8>>,
    close: RwLock<Option<oneshot::Sender<CloseFrame<'static>>>>,
    pings: RwLock<VecDeque<Duration>>,
    update: mpsc::Sender<Update>,
    state: Mutex<State>,
}

impl Session {
    pub fn new(
        addr: SocketAddr,
        send: mpsc::Sender<Vec<u8>>,
        close: oneshot::Sender<CloseFrame<'static>>,
        update: mpsc::Sender<Update>,
    ) -> Self {
        Self {
            id: SessionId(NEXT_SESSION_ID.fetch_add(1, Ordering::SeqCst)),
            addr,
            send,
            close: RwLock::new(Some(close)),
            pings: RwLock::default(),
            update,
            state: Mutex::new(State::WaitLogin),
        }
    }

    async fn add_ping(&self, ping: Duration) {
        log::debug!("Ping: {:?}", ping);
        let mut pings = self.pings.write().await;
        while pings.len() >= PING_BACKLOG_SIZE {
            pings.pop_front();
        }
        pings.push_back(ping);
    }

    pub async fn recv(&self, mut buf: &[u8]) -> Result<()> {
        use common::proto::BinRead;

        if self.closed().await {
            return Ok(());
        }

        let packet = Packet::read(&mut buf)?;

        match (self.state().await, packet) {
            (State::WaitLogin, Packet::HandshakeLogin(login)) => {
                self.update
                    .clone()
                    .send(Update::NewSession {
                        id: self.id,
                        name: login.name,
                        identity: login.identity,
                    })
                    .await?;
                self.send(&Packet::HandshakeAccept(handshake::Accept {}))
                    .await
                    .map_err(|()| anyhow::anyhow!("Channel closed"))?;
                self.set_state(State::Game).await;
                Ok(())
            }
            _ => anyhow::bail!("Client sent packet at invalid state"),
        }
    }

    async fn state(&self) -> State {
        let lock = self.state.lock().await;
        *lock
    }

    async fn set_state(&self, state: State) {
        let mut lock = self.state.lock().await;
        *lock = state;
    }

    pub async fn send(&self, packet: &Packet) -> Result<(), ()> {
        use common::proto::BinWrite;

        if self.closed().await {
            return Ok(());
        }

        let mut vec = Vec::new();
        packet.write(&mut vec);
        self.send.clone().send(vec).await.map_err(|_| ())
    }

    pub async fn closed(&self) -> bool {
        let read = self.close.read().await;
        read.is_none()
    }

    pub async fn close(&self, frame: CloseFrame<'static>) {
        let mut close = self.close.write().await;
        if let Some(close) = close.take() {
            let _ = close.send(frame);
        }
    }

    // Called form the handle fn, so no need to send anything back
    async fn set_closed(&self) {
        match self.state().await {
            State::WaitLogin => {
                self.set_state(State::Closed).await;
            }
            State::Game => {
                self.set_state(State::Closed).await;
                // if main update handler is closed, no need to send update.
                let _ = self
                    .update
                    .clone()
                    .send(Update::CloseSession { id: self.id })
                    .await;
            }
            State::Closed => {}
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        assert!(
            *self.state.get_mut() == State::Closed,
            "Unclosed session {:?}",
            self.id
        );
    }
}

pub enum Update {
    NewSession {
        id: SessionId,
        name: String,
        identity: [u8; 512 / 8],
    },
    CloseSession {
        id: SessionId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    WaitLogin,
    Game,
    Closed,
}
