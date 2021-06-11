//! Safety wrappers

#![feature(int_bits_const)]

/// Aggregate trait for all other traits in this crate
pub trait Safety: Sized {
    /// See [`LossyTrunc`][LossyTrunc].
    fn lossy_trunc<U>(self) -> U
    where
        Self: LossyTrunc<U>,
    {
        self.lossy_trunc_impl()
    }

    /// See [`SmallFloat`][SmallFloat].
    fn small_float<U>(self) -> U
    where
        Self: SmallFloat<U>,
    {
        self.small_float_impl()
    }

    /// See [`TruncInt`][TruncInt].
    fn trunc_int<U>(self) -> U
    where
        Self: TruncInt<U>,
    {
        self.trunc_int_impl()
    }
}

impl<T> Safety for T {}

/// Lossy truncation from Self to U
pub trait LossyTrunc<U>: Sized {
    /// Lossily truncate a float.
    fn lossy_trunc_impl(self) -> U;
}
impl LossyTrunc<f32> for f64 {
    fn lossy_trunc_impl(self) -> f32 {
        self as f32
    }
}

/// Converts an integer to a float probably losslessly
pub trait SmallFloat<U>: Sized {
    /// Converts an integer to a float probably losslessly
    ///
    /// # Panics
    /// Panics if the integer is too large.
    fn small_float_impl(self) -> U;
}

macro_rules! float_cast {
    ($from:ty, $to:ty) => {
        impl SmallFloat<$to> for $from {
            fn small_float_impl(self) -> $to {
                debug_assert!(
                    Self::BITS - self.leading_zeros() <= <$to>::MANTISSA_DIGITS,
                    "{} may not be losslessly converted to float",
                    self
                );
                self as $to
            }
        }
    };
}
float_cast!(i32, f32);
float_cast!(i64, f64);
float_cast!(isize, f32);
float_cast!(isize, f64);
float_cast!(u32, f32);
float_cast!(u64, f64);
float_cast!(usize, f32);
float_cast!(usize, f64);

/// Truncates a float into an integer without losing integer precision.
pub trait TruncInt<U>: Sized {
    /// Truncates a float into an integer without losing integer precision.
    ///
    /// # Panics
    /// Panics if the integer cannot hold the float value.
    fn trunc_int_impl(self) -> U;
}

macro_rules! float_trunc_int {
    ($from:ty, $to:ty) => {
        impl TruncInt<$to> for $from {
            fn trunc_int_impl(self) -> $to {
                let trunc = self.trunc();
                debug_assert!(
                    <$to>::MIN as Self <= trunc && trunc <= <$to>::MAX as Self,
                    "{} cannot fit into an integer",
                    trunc
                );
                trunc as $to
            }
        }
    };
}
float_trunc_int!(f64, u64);
float_trunc_int!(f64, u32);
float_trunc_int!(f64, usize);
float_trunc_int!(f64, i64);
float_trunc_int!(f64, i32);
float_trunc_int!(f64, isize);
float_trunc_int!(f32, u64);
float_trunc_int!(f32, u32);
float_trunc_int!(f32, usize);
float_trunc_int!(f32, i64);
float_trunc_int!(f32, i32);
float_trunc_int!(f32, isize);
