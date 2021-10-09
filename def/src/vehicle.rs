//! Vehicle definitions.

use std::ops::Range;

use codegen::Definition;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use traffloat_types::units;

use crate::atlas::Sprite;
use crate::{catalyst, lang, skill};

/// A type of vehicle.
#[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize, Definition)]
pub struct Def {
    /// ID of the vehicle type.
    #[getset(get_copy = "pub")]
    id:          Id,
    /// Name of the vehicle type.
    #[getset(get = "pub")]
    name:        lang::Item,
    /// Long description of the vehicle type.
    #[getset(get = "pub")]
    description: lang::Item,
    /// Base speed of the vehicle.
    ///
    /// Subject to terminal force and operator skill.
    #[getset(get_copy = "pub")]
    speed:       units::VehicleSpeed,
    /// The amount of cargo that the vehicle can carry.
    #[getset(get_copy = "pub")]
    capacity:    units::CargoSize,
    /// The number of non-driver inhabitants carried by the vehicle.
    #[getset(get_copy = "pub")]
    passengers:  u32,
    /// The skill required to operate this vehicle.
    #[getset(get = "pub")]
    skill:       Skill,
    /// The texture of the vehicle.
    #[getset(get = "pub")]
    texture:     Sprite,
}

/// A skill required for driving the vehicle.
#[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize, Definition)]
pub struct Skill {
    /// The skill type.
    #[getset(get = "pub")]
    skill:       skill::Id,
    /// The skill level range of varying speed multipliers.
    #[getset(get = "pub")]
    levels:      Range<units::Skill>,
    /// The multipliers applied on the driving speed.
    #[getset(get_copy = "pub")]
    multipliers: catalyst::Multipliers,
}
