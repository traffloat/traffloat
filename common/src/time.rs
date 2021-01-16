use std::ops::Mul;

use specs::WorldExt;

use crate::proto::{BinRead, BinWrite, ProtoType};
use crate::Setup;

ratio_def::units! {
    _TimeTrait(Clone + Copy);

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, codegen::Gen)] u32:
    /// Synchronized time span.
    ///
    /// The underlying float is in seconds.
    Time;
}

impl Time {
    pub fn as_secs(self) -> f32 {
        self.value() as f32
    }
}

/// The rate of change.
///
/// The inner value is the amount of change over one second.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, codegen::Gen)]
pub struct Rate<T: ProtoType + BinRead + BinWrite>(pub T);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, codegen::Gen)]
pub struct Instant(pub Time);

#[derive(Debug, Default)]
pub struct Clock {
    /// The current time
    pub now: Instant,
    /// Time since the last frame
    pub delta: Time,
}

impl<T: ProtoType + BinRead + BinWrite + Mul<f32, Output = T>> std::ops::Mul<Time> for Rate<T> {
    type Output = T;

    fn mul(self, time: Time) -> T {
        self.0 * (time.0 as f32)
    }
}

pub fn setup_specs((world, dispatcher): Setup) -> Setup {
    (world, dispatcher)
}
