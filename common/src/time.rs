//! Game clock management

use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use crate::proto::{BinRead, BinWrite, ProtoType};
use crate::Setup;

ratio_def::units! {
    /// Internal trait jjust because declarative macros are stupid.
    _TimeTrait(Clone + Copy);

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, codegen::Gen)] u32:

    /// Synchronized time span.
    ///
    /// The underlying integer is in 1/100 seconds.
    Time;
}

impl Time {
    /// Converts the time span to number of seconds.
    ///
    /// This value is not precise and shall not be used for critical logic.
    #[allow(clippy::cast_precision_loss)]
    pub fn as_secs(self) -> f32 {
        self.value() as f32 * 0.01
    }
}

/// The rate of change.
///
/// The inner value is the amount of change over one second.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, codegen::Gen)]
pub struct Rate<T: ProtoType + BinRead + BinWrite>(pub T);

/// A specific point of time,
/// represented as a duration since game epoch.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, codegen::Gen)]
pub struct Instant(pub Time);

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

impl SubAssign<Time> for Instant {
    fn sub_assign(&mut self, other: Time) {
        self.0 -= other;
    }
}

/// A resource for time read/write.
#[derive(Debug, Default)]
pub struct Clock {
    /// The current time
    pub now: Instant,
    /// Time since the last frame
    pub delta: Time,
}

impl Clock {
    /// Increases the time for the specified span.
    pub fn inc_time(&mut self, time: Time) {
        self.now += time;
        self.delta = time;
    }
}

impl<T: ProtoType + BinRead + BinWrite + Mul<f32, Output = T>> std::ops::Mul<Time> for Rate<T> {
    type Output = T;

    #[allow(clippy::cast_precision_loss)]
    fn mul(self, time: Time) -> T {
        self.0 * (time.0 as f32)
    }
}

/// Initializes the time module.
pub fn setup_specs((world, dispatcher): Setup) -> Setup {
    (world, dispatcher)
}
