use serde::{Deserialize, Serialize};
use xylem::Xylem;

use crate::{cargo, fluid, population, skill, unit};

/// A condition for a reaction.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub struct Condition {
    /// Bounds of linear interpolation.
    pub range:       ConditionRange,
    /// Effects of the condiiton.
    pub multipliers: ConditionMultipliers,
}

/// Bounds of linear interpolation of a condition.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize), serde(tag = "type"))]
pub enum ConditionRange {
    /// The condition is based on electric power surplus, but does not consume electricity.
    Electricity {
        /// Lower bound of the condition.
        #[xylem(serde(default))]
        lower: Option<unit::ElectricPower>,
        /// Upper bound of the condition.
        #[xylem(serde(default))]
        upper: Option<unit::ElectricPower>,
    },
    /// The condition is based on abundance of a cargo type in the building,
    /// but does not consume it.
    Cargo {
        /// The type of cargo.
        ty:    cargo::Id,
        #[xylem(serde(default))]
        lower: Option<unit::CargoVolume>,
        /// Upper bound of the condition.
        #[xylem(serde(default))]
        upper: Option<unit::CargoVolume>,
    },
    /// The condition is based on abundance of a fluid type in a container,
    /// but does not consume it.
    Fluid {
        /// The type of fluid.
        ty:      fluid::Id,
        /// The storage that fluid must be present.
        storage: fluid::StorageId,
        /// Lower bound of the condition.
        #[xylem(serde(default))]
        lower:   Option<unit::FluidVolume>,
        /// Upper bound of the condition.
        #[xylem(serde(default))]
        upper:   Option<unit::FluidVolume>,
    },
    /// The condition is based on the skill level of an operator inhabitant.
    /// It does not affect the skill level of the inhabitant.
    /// If there are multiple inhabitants in the container, their skill levels are summed up.
    Inhabitant {
        /// The skill required.
        skill:   skill::Id,
        /// The storage that inhabitants are assigned to.
        storage: population::StorageId,
        /// Lower bound of the condition.
        #[xylem(serde(default))]
        lower:   Option<unit::SkillLevel>,
        /// Upper bound of the condition.
        #[xylem(serde(default))]
        upper:   Option<unit::SkillLevel>,
    },
}

/// Multipliers resulting from a condiiton.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub struct ConditionMultipliers {
    ///
    pub underflow: f64,
}
