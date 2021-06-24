//! Game clock management

use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use crate::proto::{BinRead, BinWrite, ProtoType};
use crate::SetupEcs;

ratio_def::units! {
    /// Internal trait just because declarative macros are stupid.
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
    pub fn as_secs(self) -> f64 {
        self.value() as f64 * 0.01
    }
}

/// A specific point of time,
/// represented as a duration since game epoch.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, codegen::Gen)]
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

/// A resource for time read/write.
#[derive(Debug, Default, getset::CopyGetters)]
pub struct Clock {
    /// The current time
    #[getset(get_copy = "pub")]
    now: Instant, // TODO multiplayer calibration
    /// Time since the last frame
    #[getset(get_copy = "pub")]
    delta: Time,
}

impl Clock {
    /// Increases the time for the specified span.
    pub fn inc_time(&mut self, time: Time) {
        self.now += time;
        self.delta = time;
    }

    /// Sets the time to the specified instant.
    pub fn set_time(&mut self, now: Instant) {
        self.delta = now - self.now;
        self.now = now;
    }
}

/// The rate of change.
///
/// The inner value is the amount of change over one second.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, codegen::Gen)]
pub struct Rate<T: ProtoType + BinRead + BinWrite>(pub T);

impl<T: ProtoType + BinRead + BinWrite + Mul<f64, Output = T>> std::ops::Mul<Time> for Rate<T> {
    type Output = T;

    fn mul(self, time: Time) -> T {
        self.0 * (time.value() as f64)
    }
}

/// Initializes the time module.
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup
}
