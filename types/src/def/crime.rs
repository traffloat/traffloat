//! Crime definitions.

use std::ops::Range;

use smallvec::SmallVec;
use typed_builder::TypedBuilder;

use crate::def::skill;
use crate::units;

/// Consequence of a crime.
#[derive(Clone, Copy)]
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
    /// Reduce the happiness of other inhabitants.
    Antagonize(InhabitantCriterion),
    /// Set a node on fire.
    Arson,
    /// Chase other inhabitants and delete them when located in the same node.
    ///
    /// The parameter is the criteria to select inhabitants to murder.
    Murder(InhabitantCriterion),
}

/// A criterion to sort inhabitants with.
#[derive(Clone, Copy)]
pub enum InhabitantCriterion {
    /// Select the inhabitant with the highest skill.
    HighestSkill(skill::TypeId),
}

/// A type of crime customized for the game definition.
#[derive(TypedBuilder, getset::Getters, getset::CopyGetters)]
pub struct Type {
    /// Name of the crime.
    #[getset(get = "pub")]
    name: String,
    /// Description of the crime.
    #[getset(get = "pub")]
    description: String,
    /// The actual consequence of the crime.
    #[getset(get = "pub")]
    action: Action,
    /// The happiness range at which this crime may happen.
    #[getset(get = "pub")]
    trigger_happiness_range: Range<units::Happiness>,
    /// The base (unmultiplied) probability per second that an inhabitant starts to commit this
    /// crime.
    #[getset(get_copy = "pub")]
    probability: f64,
    /// The change in happiness after committing this crime.
    #[getset(get_copy = "pub")]
    happiness_change: units::Happiness,
    /// The change in skill levels after committing this crime.
    #[getset(get = "pub")]
    skill_change: SmallVec<[(skill::TypeId, units::Skill); 1]>,
}
