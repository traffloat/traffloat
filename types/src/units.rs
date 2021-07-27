//! Defines common units

use std::fmt;

use serde::{Deserialize, Serialize};

units! {
    /// A common unit type
    Unit(std::fmt::Debug + Clone + Copy + Default + PartialEq + PartialOrd);

    #[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Serialize, Deserialize)] f64:

    /// An amount of liquid.
    LiquidVolume("{} L");

    /// Viscosity of a liquid.
    LiquidViscosity("{} Pas");

    /// An absolute amount of gas.
    GasVolume("{} mol");

    /// The standard size for cargo.
    CargoSize("{} dm\u{b3}");

    /// Dynamic electricity consumed immediately.
    ElectricPower("{} W");

    /// Static electricity in stored form.
    ElectricEnergy("{} J");

    /// Orthogonal area of a node receiving sunlight.
    Brightness("{} m\u{b2}");

    /// Skill level of an inhabitant.
    Skill("{} SP");

    /// Driving force on a rail.
    RailForce("{} T");

    /// Pumping force on a liquid pipe.
    PipeForce("{} Pa");

    /// Pumping force for gas diffusion.
    FanForce("{} Pa");

    /// Speed of a vehicle on a rail.
    VehicleSpeed("{} m/s");

    /// Hitpoint of a building.
    Hitpoint("{} HP");
}

/// A unit with a maximum capacity.
#[derive(Debug, Clone, Copy, derive_new::new, getset::CopyGetters, Serialize, Deserialize)]
pub struct Portion<U: Unit> {
    /// The current value.
    #[getset(get_copy = "pub")]
    current: U,
    /// The maximum capacity.
    #[getset(get_copy = "pub")]
    max: U,
}

impl<U: Unit> Portion<U> {
    /// Initializes an empty portion.
    pub fn empty(max: U) -> Self {
        Self::new(U::default(), max)
    }

    /// Initializes a full portion.
    pub fn full(max: U) -> Self {
        Self::new(max, max)
    }
}

impl<U: Unit + fmt::Display> fmt::Display for Portion<U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} / {}", self.current, self.max)
    }
}
