//! Protocol-related types

pub mod handshake;
mod io;

/// A large prime number used for polynomial checksum.
pub const PROTO_TYPE_CKSUM_PRIME: u128 = (1_u128 << 80) - 65;

pub use io::{BinRead, BinWrite, Error, ProtoType};

/// Wrapper for all packets.
#[derive(codegen::Gen)]
pub enum Packet {
    /// [`handshake::Login`](handshake/struct.Login.html)
    HandshakeLogin(handshake::Login),
    /// [`handshake::Accept`](handshake/struct.Accept.html)
    HandshakeAccept(handshake::Accept),
    /// [`handshake::Reject`](handshake/struct.Reject.html)
    HandshakeReject(handshake::Reject),
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
