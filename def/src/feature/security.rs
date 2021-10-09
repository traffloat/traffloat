//! Security-related node features.

use codegen::Definition;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use traffloat_types::units;

use crate::catalyst::Catalyst;
use crate::skill;

/// A security policy affecting entry/exit.
#[derive(Debug, Clone, CopyGetters, Getters, Serialize, Deserialize, Definition)]
pub struct Policy {
    /// The catalysts affecting the breach probability.
    #[getset(get = "pub")]
    catalysts: SmallVec<[Catalyst; 2]>,

    /// The skill type to check.
    #[getset(get = "pub")]
    skill:   skill::Id,
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Definition)]
pub enum SkillRequirement {
    /// Minimum skill level.
    AtLeast(units::Skill),
    /// Maximum skill level.
    AtMost(units::Skill),
}
