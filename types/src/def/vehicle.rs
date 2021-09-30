//! Vehicle definitions.

use std::ops::Range;

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::def::{catalyst, skill};
use crate::units;

/// Identifies a vehicle type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TypeId(pub ArcStr);

/// A type of vehicle.
#[derive(
    Debug, Clone, TypedBuilder, getset::Getters, getset::CopyGetters, Serialize, Deserialize,
)]
pub struct Type {
    /// Name of the vehicle type.
    #[getset(get = "pub")]
    name:         ArcStr,
    /// Long description of the vehicle type.
    #[getset(get = "pub")]
    description:  ArcStr,
    /// Base speed of the vehicle.
    ///
    /// Subject to terminal force and operator skill.
    #[getset(get_copy = "pub")]
    speed:        units::VehicleSpeed,
    /// The amount of cargo that the vehicle can carry.
    #[getset(get_copy = "pub")]
    capacity:     units::CargoSize,
    /// The number of non-driver inhabitants carried by the vehicle.
    #[getset(get_copy = "pub")]
    passengers:   u32,
    /// The skill required to operate this vehicle.
    #[getset(get = "pub")]
    skill:        Skill,
    /// The texture source path of the vehicle.
    #[getset(get = "pub")]
    texture_src:  ArcStr,
    /// The texture name of the vehicle.
    #[getset(get = "pub")]
    texture_name: ArcStr,
}

/// A skill required for driving the vehicle.
#[derive(
    Debug, Clone, TypedBuilder, getset::Getters, getset::CopyGetters, Serialize, Deserialize,
)]
pub struct Skill {
    /// The skill type.
    #[getset(get = "pub")]
    skill:       skill::TypeId,
    /// The skill level range of varying speed multipliers.
    #[getset(get = "pub")]
    levels:      Range<units::Skill>,
    /// The multipliers applied on the driving speed.
    #[getset(get_copy = "pub")]
    multipliers: catalyst::Multipliers,
}
