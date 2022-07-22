use serde::{Deserialize, Serialize};
use xylem::Xylem;

use crate::{cargo, condition, fluid, population, skill, unit};

/// A reaction that happens in a specific building type.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub struct Reaction {
    /// The copy-safe identifier.
    #[xylem(args(new = true))]
    pub id:     Id,
    /// The string identifier.
    #[xylem(serde(default))]
    pub id_str: IdString,

    /// The conditions affecting the rate of the reaction.
    pub conditions: Vec<condition::Condition>,
    /// Inputs and outputs of the reaction.
    ///
    /// The availability of input materials and output capacity also serve as conditions.
    pub puts:       Vec<Put>,
}

impl_identifiable!(Reaction);

/// Input or output of a reaction.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub enum Put {
    /// Generates/consumes electricity.
    Electricity {
        /// Amount of electricity generated.
        power: unit::ElectricPower,
    },
    /// Produces/consumes cargo.
    Cargo {
        /// Type of cargo produced.
        ty:     cargo::Id,
        /// Amount of cargo produced.
        volume: unit::CargoVolume,
    },
    /// Produces/consumes fluid.
    Fluid {
        /// Type of fluid produced.
        ty:      fluid::Id,
        /// The storage where the fluid is taken from or added to.
        storage: fluid::StorageId,
        /// Amount of fluid produced.
        volume:  unit::FluidVolume,
    },
    Skill {
        /// Skill type affected.
        ty:      skill::Id,
        /// The storage where impacted inhabitants are located in.
        storage: population::StorageId,
        /// Change in skill level.
        level:   unit::SkillLevel,
    },
}
