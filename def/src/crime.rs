//! Crime definitions.

use std::ops::Range;

use codegen::{Definition, IdStr};
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use traffloat_types::units;

use crate::{lang, skill};

/// A type of crime customized for the game definition.
#[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize, Definition)]
pub struct Def {
    /// ID of the crime type.
    #[getset(get_copy = "pub")]
    id:           Id,
    /// String ID of the crime type.
    #[getset(get = "pub")]
    id_str:       IdStr,
    /// Name of the crime.
    #[getset(get = "pub")]
    name:         lang::Item,
    /// Description of the crime.
    #[getset(get = "pub")]
    description:  lang::Item,
    /// The actual consequence of the crime.
    #[getset(get = "pub")]
    action:       Action,
    /// The triggering condition for the crime.
    #[getset(get = "pub")]
    trigger:      TriggerSkill,
    /// The change in skill levels after committing this crime.
    #[getset(get = "pub")]
    skill_change: SmallVec<[SkillChange; 1]>,
}

/// Consequence of a crime.
#[derive(Debug, Clone, Serialize, Deserialize, Definition)]
#[serde(tag = "type")]
pub enum Action {
    /// Steal random cargo carried by inhabitants in the same node or vehicle.
    ///
    /// The parameter is the maximum amount of cargo that the inhabitant may steal.
    InhabitantTheft {
        /// The amount stolen
        amount: units::CargoSize,
    },
    /// Steal random cargo carried by vehicles parked in nodes.
    ///
    /// The parameter is the maximum amount of cargo that the inhabitant may steal.
    VehicleTheft {
        /// The amount stolen
        amount: units::CargoSize,
    },
    /// Steal random cargo stored in a node.
    ///
    /// The parameter is the maximum amount of cargo that the inhabitant may steal.
    NodeTheft {
        /// The amount stolen
        amount: units::CargoSize,
    },
    // /// Reduce the skill level of other inhabitants.
    // Antagonize(InhabitantCriterion, skill::TypeId, units::Skill),
    /// Set a node on fire.
    Arson,
}

// /// A criterion to sort inhabitants with.
// #[derive(Debug, Clone, Serialize, Deserialize, Definition)]
// #[serde(tag = "type")]
// pub enum InhabitantCriterion {
// /// Select the inhabitant with the highest skill.
// HighestSkill(skill::TypeId),
// }

/// Triggering condition for a crime.
#[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize, Definition)]
pub struct TriggerSkill {
    /// The skill type to trigger the crime.
    #[getset(get_copy = "pub")]
    ty:          skill::Id,
    /// The skill range at which this crime may happen.
    #[getset(get = "pub")]
    range:       Range<units::Skill>,
    /// The base (unmultiplied) probability per second that an inhabitant starts to commit this
    /// crime.
    #[getset(get_copy = "pub")]
    probability: f64,
}

/// A change in skill level.
#[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize, Definition)]
pub struct SkillChange {
    /// The skill type to change.
    #[getset(get_copy = "pub")]
    skill:  skill::Id,
    /// The amount changed.
    #[getset(get_copy = "pub")]
    change: units::Skill,
}
