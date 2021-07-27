//! Vehicle definitions.

use std::ops::Range;

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::def::{reaction, skill};
use crate::units;

/// Identifies a vehicle category
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeId(pub usize);

/// A type of vehicle.
#[derive(
    Debug, Clone, TypedBuilder, getset::Getters, getset::CopyGetters, Serialize, Deserialize,
)]
pub struct Type {
    /// Name of the vehicle type.
    #[getset(get = "pub")]
    name: ArcStr,
    /// Long description of the vehicle type.
    #[getset(get = "pub")]
    description: ArcStr,
    /// Base speed of the vehicle.
    ///
    /// Subject to terminal force and operator skill.
    #[getset(get_copy = "pub")]
    speed: units::VehicleSpeed,
    /// The amount of cargo that the vehicle can carry.
    #[getset(get_copy = "pub")]
    capacity: units::CargoSize,
    /// The number of non-driver inhabitants carried by the vehicle.
    #[getset(get_copy = "pub")]
    passengers: u32,
    /// The skill required to operate this vehicle.
    #[getset(get = "pub")]
    skill: Skill,
    /// Name of the texture.
    #[getset(get = "pub")]
    texture: ArcStr,
}

/// A skill required for driving the vehicle.
#[derive(
    Debug, Clone, TypedBuilder, getset::Getters, getset::CopyGetters, Serialize, Deserialize,
)]
pub struct Skill {
    /// The skill type.
    #[getset(get_copy = "pub")]
    skill: skill::TypeId,
    /// The skill level range of varying speed multipliers.
    #[getset(get = "pub")]
    levels: Range<units::Skill>,
    /// The multipliers applied on the driving speed.
    #[getset(get_copy = "pub")]
    multipliers: reaction::Multipliers,
}
