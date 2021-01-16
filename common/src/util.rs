/// The common port
pub const DEFAULT_PORT: u16 = 15384;
/// A string representation of the common port.
pub const DEFAULT_PORT_STR: &str = "15384";

/// Checks whether the client name is valid.
pub fn is_valid_name(name: &str) -> bool {
    name.trim().len() >= 3 && name.len() <= 31
}
