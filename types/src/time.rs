//! Chronological units

use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

units! {
    /// Internal trait just because declarative macros are stupid.
    _TimeTrait(Clone + Copy);

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)] u32:

    /// Synchronized time span.
    ///
    /// The underlying integer is in 1/100 seconds.
    Time("{:1} s");
}

impl Time {
    /// Converts the time span to number of seconds.
    ///
    /// This value is not precise and shall not be used for critical logic.
    pub fn as_secs(self) -> f64 {
        self.value() as f64 * 0.01
    }

    /// An empty interval.
    pub fn zero() -> Self {
        Self(0)
    }

    /// Returns the integer quotient of the two time spans.
    pub fn int_div(self, other: Self) -> u32 {
        self.0 / other.0
    }
}

/// A specific point of time,
/// represented as a duration since game epoch.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(pub Time);

impl Instant {
    /// Returns the time since epoch
    pub fn since_epoch(self) -> Time {
        self.0
    }
}

impl Add<Time> for Instant {
    type Output = Self;

    fn add(self, other: Time) -> Self {
        Self(self.0 + other)
    }
}

impl AddAssign<Time> for Instant {
    fn add_assign(&mut self, other: Time) {
        self.0 += other;
    }
}

impl Sub<Time> for Instant {
    type Output = Self;

    fn sub(self, other: Time) -> Self {
        Self(self.0 - other)
    }
}

impl Sub<Instant> for Instant {
    type Output = Time;

    fn sub(self, other: Self) -> Time {
        self.0 - other.0
    }
}

impl SubAssign<Time> for Instant {
    fn sub_assign(&mut self, other: Time) {
        self.0 -= other;
    }
}

/// The rate of change.
///
/// The inner value is the amount of change over one second.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Rate<T>(pub T);

impl<T: Mul<f64, Output = T>> std::ops::Mul<Time> for Rate<T> {
    type Output = T;

    fn mul(self, time: Time) -> T {
        self.0 * (time.value() as f64)
    }
}
