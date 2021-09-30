use std::{cmp, ops};

/// The common port
pub const DEFAULT_PORT: u16 = 15384;

/// Checks whether the client name is valid.
pub fn is_valid_name(name: &str) -> bool { name.trim().len() >= 3 && name.len() <= 31 }

/// A f64 wrapper that is guaranteed to be finite.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Finite(f64);

impl Finite {
    /// Constructs a Finite struct.
    ///
    /// # Panic
    /// Panics if the parameter is not finite.
    pub fn new(f: f64) -> Self {
        assert!(f.is_finite(), "Attempt to create Finite with non-finite float");
        Self(f)
    }

    /// Retrieves the underlying value.
    pub fn value(self) -> f64 { self.0 }
}

impl Eq for Finite {}

#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for Finite {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).expect("Values should be finite")
    }
}

/// Linear interpolation from a to b, with ratio=0 as a and ratio=1 as b
pub fn lerp<T, U>(a: T, b: T, ratio: U) -> T
where
    T: Copy + ops::Add<Output = T> + ops::Sub<Output = T> + ops::Mul<U, Output = T>,
{
    a + (b - a) * ratio
}

/// Checks if a value is different from the default of its type.
///
/// Used in serde fields: `#[serde(skip_serializing_if = "crate::is_default")]`
#[inline(always)]
pub fn is_default<T: PartialEq + Default>(value: &T) -> bool { *value == T::default() }
