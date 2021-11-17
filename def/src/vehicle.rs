//! Vehicle definitions.

use std::ops::Range;

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use traffloat_types::units;

use crate::atlas::IconRef;
use crate::{catalyst, lang, skill, IdString};

/// Identifies a vehicle type.
pub type Id = crate::Id<Def>;

impl_identifiable!(Def);

/// A type of vehicle.
#[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct Def {
    /// ID of the vehicle type.
    #[getset(get_copy = "pub")]
    #[cfg_attr(feature = "xy", xylem(args(new = true)))]
    id:          Id,
    /// String ID of the vehicle type.
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    id_str:      IdString<Def>,
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
    skill:       Vec<Skill>,
    /// The texture of the vehicle.
    #[getset(get = "pub")]
    texture:     IconRef,
}

/// A skill required for driving the vehicle.
#[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
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
