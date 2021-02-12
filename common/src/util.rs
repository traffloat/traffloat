use std::cmp;

/// The common port
pub const DEFAULT_PORT: u16 = 15384;
/// A string representation of the common port.
pub const DEFAULT_PORT_STR: &str = "15384";

/// Checks whether the client name is valid.
pub fn is_valid_name(name: &str) -> bool {
    name.trim().len() >= 3 && name.len() <= 31
}

/// A f64 wrapper that is guaranteed to be finite.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Finite(f64);

impl Finite {
    /// Constructs a Finite struct.
    ///
    /// # Panic
    /// Panics if the parameter is not finite.
    pub fn new(f: f64) -> Self {
        assert!(
            f.is_finite(),
            "Attempt to create Finite with non-finite float"
        );
        Self(f)
    }

    /// Retrieves the underlying value.
    pub fn value(self) -> f64 {
        self.0
    }
}

impl Eq for Finite {}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for Finite {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(&other).expect("Values should be finite")
    }
}
