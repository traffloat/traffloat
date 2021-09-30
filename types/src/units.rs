//! Defines common units

use std::fmt;

use serde::{Deserialize, Serialize};

units! {
    /// A common unit type
    Unit(std::fmt::Debug + Clone + Copy + Default + PartialEq + PartialOrd);

    #[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Serialize, Deserialize)] f64:

    /// An amount of liquid.
    LiquidVolume("", " L")(round:round);

    /// Viscosity of a liquid.
    LiquidViscosity("", " Pa.s")(round:round);

    /// An absolute amount of gas.
    GasVolume("", " mol")(round:round);

    /// The standard size for cargo.
    CargoSize("", " dm\u{b3}")(round:round);

    /// Dynamic electricity consumed immediately.
    ElectricPower("", " W")(round:round);

    /// Static electricity in stored form.
    ElectricEnergy("", " J")(round:round);

    /// Orthogonal area of a node receiving sunlight.
    Brightness("", " m\u{b2}")(round:round);

    /// Skill level of an inhabitant.
    Skill("", "pt")(round:round);

    /// Driving force on a rail.
    RailForce("", " T")(round:round);

    /// Pumping force on a liquid pipe.
    PipeForce("", " Pa")(round:round);

    /// Pumping force for gas diffusion.
    FanForce("", " Pa")(round:round);

    /// Speed of a vehicle on a rail.
    VehicleSpeed("", " m/s")(round:round);

    /// Hitpoint of a building.
    Hitpoint("", " HP")(round:round);
}

/// A unit with a maximum capacity.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    derive_new::new,
    getset::CopyGetters,
    Serialize,
    Deserialize,
)]
pub struct Portion<U: Unit> {
    /// The current value.
    #[getset(get_copy = "pub")]
    current: U,
    /// The maximum capacity.
    #[getset(get_copy = "pub")]
    max:     U,
}

impl<U: Unit> Portion<U> {
    /// Initializes an empty portion.
    pub fn empty(max: U) -> Self { Self::new(U::default(), max) }

    /// Initializes a full portion.
    pub fn full(max: U) -> Self { Self::new(max, max) }

    /// The filled ratio of the portion.
    pub fn ratio(self) -> f64 { self.current.value() / self.max.value() }
}

impl<U: Unit + fmt::Display> fmt::Display for Portion<U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} / {}", self.current, self.max)
    }
}

/// A unit that can be rounded off.
pub trait RoundedUnit {
    /// Round off the unit.
    ///
    /// The precision is the number of decimal places.
    /// A negative precision means the number of zeros.
    fn round(self, precision: i32) -> Self;
}
