//! Safety wrappers

/// Aggregate trait for all other traits in this crate
pub trait Safety: Sized {
    /// See [`LossyTrunc`].
    fn lossy_trunc<U>(self) -> U
    where
        Self: LossyTrunc<U>,
    {
        self.lossy_trunc_impl()
    }

    /// See [`SmallFloat`].
    fn small_float<U>(self) -> U
    where
        Self: SmallFloat<U>,
    {
        self.small_float_impl()
    }

    /// See [`TruncInt`].
    fn trunc_int<U>(self) -> U
    where
        Self: TruncInt<U>,
    {
        self.trunc_int_impl()
    }

    /// See [`SmallUnsigned`].
    fn homosign<U>(self) -> U
    where
        Self: SmallUnsigned<U>,
    {
        self.small_unsigned()
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

/// Flips a small unsigned number between `ixx` and `uxx`.
pub trait SmallUnsigned<U>: Sized {
    /// Flips a small unsigned number between `ixx` and `uxx`.
    ///
    /// # Panics
    /// Panics if the integer has the MSB set.
    fn small_unsigned(self) -> U;
}

macro_rules! small_unsigned_int {
    ($from:ty, $to:ty) => {
        impl SmallUnsigned<$to> for $from {
            #[allow(unused_comparisons)]
            fn small_unsigned(self) -> $to {
                let target = self as $to;
                debug_assert!(
                    self >= 0 && target >= 0,
                    "{} is not homogeneous over signs",
                    self,
                );
                target
            }
        }
    };
}

small_unsigned_int!(u8, i8);
small_unsigned_int!(i8, u8);
small_unsigned_int!(u16, i16);
small_unsigned_int!(i16, u16);
small_unsigned_int!(u32, i32);
small_unsigned_int!(i32, u32);
small_unsigned_int!(u64, i64);
small_unsigned_int!(i64, u64);
small_unsigned_int!(usize, isize);
small_unsigned_int!(isize, usize);
