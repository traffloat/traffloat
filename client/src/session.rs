use std::convert::TryInto;

use anyhow::Context;
use yew::services::websocket::WebSocketTask;

use common::proto::{handshake, BinRead, BinWrite, Packet};

#[derive(derive_new::new)]
pub struct Session {
    #[new(value = "Step::SecureOpen")]
    step: Step,
    allow_insecure: bool,
    pub ws: WebSocketTask,
    name: String,
    identity: Vec<u8>,
}

impl Session {
    pub fn handle_opened(&mut self) {
        match self.step {
            Step::SecureOpen | Step::InsecureOpen => {
                self.step = Step::WaitingVersion;
                self.ws.send_binary(Ok(common::proto::VERSION.to_vec()));
            }
            _ => unreachable!("Socket opened multiple times"),
        }
    }

    pub fn handle_error(&mut self) -> ErrorHandler {
        match self.step {
            Step::SecureOpen => {
                self.step = Step::InsecureOpen;
                ErrorHandler::RetryInsecure
            }
            _ => ErrorHandler::Close,
        }
    }

    pub fn handle_closed(&mut self) {}

    pub fn handle_message(
        &mut self,
        payload: &[u8],
        mut logger: impl FnMut(String),
    ) -> anyhow::Result<Option<common::proto::Packet>> {
        match self.step {
            Step::SecureOpen | Step::InsecureOpen => {
                anyhow::bail!("Unexpected packet received before handshake is sent");
            }
            Step::WaitingVersion => {
                anyhow::ensure!(
                    payload == common::proto::VERSION,
                    "Incompatible server version {} and client version {}",
                    hex::encode(payload),
                    hex::encode(&common::proto::VERSION),
                );

                logger(String::from(
                    "\u{2713} Server version is compatible, logging in",
                ));
                self.step = Step::WaitingHandshake;

                self.send_packet(Packet::HandshakeLogin(handshake::Login {
                    identity: self.identity.clone().try_into().expect("sha512 output"),
                    name: self.name.clone(),
                }));

                Ok(None)
            }
            Step::WaitingHandshake => {
                let mut payload_copy = payload;
                let packet = Packet::read(&mut payload_copy).with_context(|| {
                    format!("Malformed packet payload {}", hex::encode(payload))
                })?;
                match packet {
                    Packet::HandshakeReject(packet) => {
                        anyhow::bail!("Login rejected: {}", &packet.reason)
                    }
                    Packet::HandshakeAccept(_) => (),
                    _ => anyhow::bail!("Received invalid packet during handshake"),
                }

                logger(String::from("Server accepted login request"));
                self.step = Step::Downloading;
                Ok(None)
            }
            Step::Downloading | Step::Playing => {
                let mut payload_copy = payload;
                let packet = Packet::read(&mut payload_copy).with_context(|| {
                    format!("Malformed packet payload {}", hex::encode(payload))
                })?;

                if let Packet::GameStart(_) = &packet {
                    logger(String::from("Starting game"));
                    self.step = Step::Playing;
                }
                Ok(Some(packet))
            }
        }
    }

    pub fn send_packet(&mut self, packet: common::proto::Packet) {
        let mut vec = Vec::new();
        packet.write(&mut vec);
        self.ws.send_binary(Ok(vec));
    }
}

#[derive(Debug, Clone, Copy)]
enum Step {
    SecureOpen,
    InsecureOpen,
    WaitingVersion,
    WaitingHandshake,
    Downloading,
    Playing,
}

pub enum ErrorHandler {
    // Report,
    Close,
    RetryInsecure,
}
