//! Crime definitions.

use std::ops::Range;

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use typed_builder::TypedBuilder;

use crate::def::skill;
use crate::units;

/// Consequence of a crime.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Action {
    /// Steal random cargo carried by inhabitants in the same node or vehicle.
    ///
    /// The parameter is the maximum amount of cargo that the inhabitant may steal.
    InhabitantTheft(units::CargoSize),
    /// Steal random cargo carried by vehicles parked in nodes.
    ///
    /// The parameter is the maximum amount of cargo that the inhabitant may steal.
    VehicleTheft(units::CargoSize),
    /// Steal random cargo stored in a node.
    ///
    /// The parameter is the maximum amount of cargo that the inhabitant may steal.
    NodeTheft(units::CargoSize),
    /// Reduce the skill level of other inhabitants.
    Antagonize(InhabitantCriterion, skill::TypeId, units::Skill),
    /// Set a node on fire.
    Arson,
}

/// A criterion to sort inhabitants with.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum InhabitantCriterion {
    /// Select the inhabitant with the highest skill.
    HighestSkill(skill::TypeId),
}

/// A type of crime customized for the game definition.
#[derive(TypedBuilder, getset::Getters, getset::CopyGetters, Serialize, Deserialize)]
pub struct Type {
    /// Name of the crime.
    #[getset(get = "pub")]
    name: ArcStr,
    /// Description of the crime.
    #[getset(get = "pub")]
    description: ArcStr,
    /// The actual consequence of the crime.
    #[getset(get = "pub")]
    action: Action,
    /// The skill type to trigger the crime.
    #[getset(get_copy = "pub")]
    trigger_skill: skill::TypeId,
    /// The skill range at which this crime may happen.
    #[getset(get = "pub")]
    trigger_skill_range: Range<units::Skill>,
    /// The base (unmultiplied) probability per second that an inhabitant starts to commit this
    /// crime.
    #[getset(get_copy = "pub")]
    probability: f64,
    /// The change in skill levels after committing this crime.
    #[getset(get = "pub")]
    skill_change: SmallVec<[(skill::TypeId, units::Skill); 1]>,
}
