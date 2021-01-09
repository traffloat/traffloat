//! Handshake packets

/// Sends login information
#[derive(codegen::Gen)]
pub struct Login {
    /// The client identity unique to this server.
    pub identity: [u8; 512 / 8],
    /// The client display name.
    pub name: String,
}

/// Notifies the client that the login is accepted.
#[derive(codegen::Gen)]
pub struct Accept {}

/// Notifies the client that the login is rejected.
#[derive(codegen::Gen)]
pub struct Reject {
    /// The reason for login rejection.
    pub reason: String,
}
