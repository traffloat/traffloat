use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

use sha2::Digest;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus};

use super::WebSocket;
use common::proto::{handshake, BinRead, BinWrite, Packet};

pub struct Connect {
    link: ComponentLink<Self>,
    props: Properties,
    status_log: Vec<String>,
    ws: WebSocket,
    step: Step,
}

impl Component for Connect {
    type Message = Message;
    type Properties = Properties;

    fn create(props: Properties, link: ComponentLink<Self>) -> Self {
        let addr = format!("wss://{}:{}", props.addr, props.port);

        let ws = Rc::new(RefCell::new(
            WebSocketService::connect_binary(
                &addr,
                link.callback(Message::WsReceive),
                link.callback(Message::WsStatus),
            )
            .unwrap(),
        ));

        Self {
            link,
            props,
            status_log: vec![format!("Connecting to {}", addr)],
            ws,
            step: Step::SecureConnect,
        }
    }

    fn update(&mut self, msg: Message) -> ShouldRender {
        match msg {
            Message::WsStatus(status) => match status {
                WebSocketStatus::Error => match self.step {
                    Step::SecureConnect if self.props.allow_insecure => {
                        // reconnect
                        self.step = Step::InsecureConnect;
                        let addr = format!("ws://{}:{}", self.props.addr, self.props.port);
                        self.ws = Rc::new(RefCell::new(
                            WebSocketService::connect_binary(
                                &addr,
                                self.link.callback(Message::WsReceive),
                                self.link.callback(Message::WsStatus),
                            )
                            .unwrap(),
                        ));
                        self.status_log.push(
                            "Secure connection failed, trying insecure connection".to_owned(),
                        );
                        true
                    }
                    Step::SecureConnect | Step::InsecureConnect => {
                        // connection failed
                        self.props
                            .error_hook
                            .emit(Some("Connection failed".to_owned()));
                        false
                    }
                    _ => {
                        // connection broken
                        self.props
                            .error_hook
                            .emit(Some("A network error occurred".to_owned()));
                        false
                    }
                },
                WebSocketStatus::Opened => {
                    // TODO
                    self.status_log.push("Connection established".to_owned());
                    self.step = Step::VersionHandshake;

                    let checksum = common::proto::VERSION;
                    let mut ws = self.ws.borrow_mut();
                    ws.send_binary(Ok(checksum.to_vec()));
                    true
                }
                WebSocketStatus::Closed => {
                    self.props
                        .error_hook
                        .emit(Some("Connection closed".to_owned()));
                    true
                }
            },
            Message::WsReceive(recv) => {
                let ret: anyhow::Result<bool> = (|| {
                    let recv = recv?;
                    Ok(match self.step {
                        Step::VersionHandshake => {
                            if recv[..] != common::proto::VERSION {
                                let error = format!(
                            "Incompatible protocol versions (client = {:032x?}, server = {:032x?})",
                            &common::proto::VERSION[..],
                            &recv[..],
                        );
                                self.props.error_hook.emit(Some(error));
                                return Ok(true);
                            }

                            let mut ws = self.ws.borrow_mut();
                            ws.send_binary(Ok(Packet::HandshakeLogin(handshake::Login {
                                identity: self.props.hashed_identity(),
                                name: self.props.name.clone(),
                            })
                            .write_to_vec()));

                            self.status_log.push("Logging in".to_owned());

                            self.step = Step::AcceptReject;
                            true
                        }
                        Step::AcceptReject => {
                            let packet = Packet::read(&mut &recv[..]);

                            match packet {
                                Ok(Packet::HandshakeAccept(_)) => {
                                    log::debug!("Handshake accepted");
                                }
                                Ok(Packet::HandshakeReject(packet)) => {
                                    anyhow::bail!("Connection rejected: {}", &packet.reason)
                                }
                                _ => anyhow::bail!("Server responded with invalid data"),
                            }

                            self.props.ready_hook.emit(super::GameArgs {
                                ws: Rc::clone(&self.ws),
                            });
                            true
                        }
                        step => {
                            log::warn!("Received message at unexpected step {:?}", step);
                            false
                        }
                    })
                })();
                match ret {
                    Ok(r) => r,
                    Err(err) => {
                        self.props.error_hook.emit(Some(err.to_string()));
                        true
                    }
                }
            }
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        unreachable!()
    }

    fn view(&self) -> Html {
        html! {
            <div style="max-width: 640px; margin: 0 auto;">
                <h1>{ "traffloat" }</h1>
                <h2>{ "Connecting\u{2026}" }</h2>
                <ul>{ for self.status_log.iter().map(|status| html! {
                    <li>{ status }</li>
                }) } </ul>
            </div>
        }
    }
}

pub enum Message {
    WsStatus(WebSocketStatus),
    WsReceive(anyhow::Result<Vec<u8>>),
}

#[derive(Clone, Properties)]
pub struct Properties {
    pub addr: String,
    pub port: u16,
    pub allow_insecure: bool,
    pub identity: Vec<u8>,
    pub name: String,
    pub ready_hook: Callback<super::GameArgs>,
    pub error_hook: Callback<Option<String>>,
}

impl Properties {
    pub fn hashed_identity(&self) -> [u8; 512 / 8] {
        let mut sha = sha2::Sha512::new();
        sha.update(&self.identity[..]);
        sha.update(self.addr.as_bytes());
        sha.update(&self.port.to_le_bytes());
        let array = sha.finalize();
        array.as_slice().try_into().expect("512 / 8 == 64")
    }
}

#[derive(Debug, Clone, Copy)]
enum Step {
    SecureConnect,
    InsecureConnect,
    VersionHandshake,
    AcceptReject,
}
