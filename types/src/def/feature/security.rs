//! Security-related node features.

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use typed_builder::TypedBuilder;

use crate::def::catalyst::Catalyst;
use crate::def::skill;
use crate::units;

/// A security policy affecting entry/exit.
#[derive(
    Debug, Clone, TypedBuilder, getset::CopyGetters, getset::Getters, Serialize, Deserialize,
)]
pub struct Policy {
    /// The catalysts affecting the breach probability.
    #[getset(get = "pub")]
    catalysts: SmallVec<[Catalyst; 2]>,

    /// The skill type to check.
    #[getset(get = "pub")]
    skill: skill::TypeId,
    /// The constraints on skill level to deny entry/exit.
    #[getset(get_copy = "pub")]
    deny_if: SkillRequirement,

    /// The probability per second per inhabitant that
    /// the inhabitant has lower skill level than required
    /// but still can enter/exit the building.
    #[getset(get = "pub")]
    breach_probability: f64,
}

/// A requirement of skill level.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SkillRequirement {
    /// Minimum skill level.
    AtLeast(units::Skill),
    /// Maximum skill level.
    AtMost(units::Skill),
}
