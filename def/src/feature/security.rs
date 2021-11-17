//! Security-related node features.

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use traffloat_types::units;

use crate::catalyst::Catalyst;
use crate::skill;

/// A security policy affecting entry/exit.
#[derive(Debug, Clone, CopyGetters, Getters, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct Policy {
    /// The catalysts affecting the breach probability.
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    catalysts: SmallVec<[Catalyst; 2]>,

    /// The constraints on skill level to deny entry/exit.
    ///
    /// Multiple requirements are joined with an OR operator.
    #[getset(get = "pub")]
    deny_if: Vec<SkillRequirement>,

    /// The probability per second per inhabitant that
    /// the inhabitant has lower skill level than required
    /// but still can enter/exit the building.
    #[getset(get = "pub")]
    breach_probability: f64,
}

/// A requirement of skill level.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize), serde(tag = "type")))]
pub enum SkillRequirement {
    /// Minimum skill level.
    AtLeast {
        /// The skill type to check.
        skill: skill::Id,
        /// The minimum skill level.
        level: units::Skill,
    },
    /// Maximum skill level.
    AtMost {
        /// The skill type to check.
        skill: skill::Id,
        /// The maximum skill level.
        level: units::Skill,
    },
}
