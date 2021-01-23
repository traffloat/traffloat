//! Protocol-related types

pub mod handshake;
mod io;
pub mod objects;

/// A large prime number used for polynomial checksum.
pub const PROTO_TYPE_CKSUM_PRIME: u128 = (1_u128 << 80) - 65;

pub use io::{BinRead, BinWrite, Error, ProtoType};

/// Wrapper for all packets.
#[allow(missing_docs)]
#[derive(codegen::Gen)]
pub enum Packet {
    HandshakeLogin(handshake::Login),
    HandshakeAccept(handshake::Accept),
    HandshakeReject(handshake::Reject),
    AddNodes(objects::AddNodes),
    GameStart(handshake::GameStart),
}

/// The computed version checksum
pub const VERSION: [u8; 16] = Packet::CHECKSUM.to_le_bytes();

#[cfg(feature = "yew")]
impl Into<yew::format::Binary> for Packet {
    fn into(self) -> yew::format::Binary {
        let mut vec = Vec::new();
        <Self as BinWrite>::write(&self, &mut vec);
        Ok(vec)
    }
}
