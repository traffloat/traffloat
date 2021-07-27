//! Building definitions

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::{reaction, skill};
use crate::space::Matrix;
use crate::{geometry, units};

/// Identifies a building category
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeId(pub usize);

/// A type of building.
#[derive(
    Debug, Clone, TypedBuilder, getset::CopyGetters, getset::Getters, Serialize, Deserialize,
)]
pub struct Type {
    /// Name of the building type.
    #[getset(get = "pub")]
    name: ArcStr,
    /// Short summary of the building type.
    #[getset(get = "pub")]
    summary: ArcStr,
    /// Long description of the building type.
    #[getset(get = "pub")]
    description: ArcStr,
    /// Category of the building type.
    #[getset(get_copy = "pub")]
    category: CategoryId,
    /// Shape of the building.
    #[getset(get = "pub")]
    shape: Shape,
    /// Reactions associated with the building.
    #[getset(get = "pub")]
    reactions: Vec<(reaction::TypeId, ReactionPolicy)>,
    /// Maximum hitpoint of a building.
    ///
    /// The actual hitpoint is subject to asteroid and fire damage.
    /// It can be restored by construction work.
    #[getset(get = "pub")]
    hitpoint: units::Hitpoint,
    /// Storage provided by a building
    #[getset(get = "pub")]
    storage: Storage,
    /// Extra features associated with the building.
    #[getset(get = "pub")]
    features: Vec<ExtraFeature>,
}

/// Shape of a building.
#[derive(
    Debug, Clone, TypedBuilder, getset::CopyGetters, getset::Getters, Serialize, Deserialize,
)]
pub struct Shape {
    /// The unit model type.
    #[getset(get_copy = "pub")]
    unit: geometry::Unit,
    /// The transformation matrix from the unit model to this shape.
    #[getset(get_copy = "pub")]
    transform: Matrix,
    /// The texture source path of the building.
    #[getset(get = "pub")]
    texture_src: ArcStr,
    /// The texture name of the building.
    #[getset(get = "pub")]
    texture_name: ArcStr,
}

/// Reaction behaviour specific to this building.
#[derive(Debug, Clone, TypedBuilder, getset::CopyGetters, Serialize, Deserialize)]
#[builder(field_defaults(default))]
pub struct ReactionPolicy {
    /// Whethre the reaction rate can be configured by the players.
    #[get_copy = "pub"]
    configurable: bool,
    /// What happens when inputs underflow.
    #[get_copy = "pub"]
    on_underflow: FlowPolicy,
    /// What happens when outputs overflow.
    #[get_copy = "pub"]
    on_overflow: FlowPolicy,
}

/// behaviour when inputs underflow or outputs overflow.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FlowPolicy {
    /// Reduce the rate of reaction such that the input/output capacity is just enough.
    ReduceRate,
}

impl Default for FlowPolicy {
    fn default() -> Self {
        Self::ReduceRate
    }
}

/// Storage provided by a building.
///
/// This storage is also used as a buffer for liquid and gas transfer.
/// The storage size is the maximum amount of liquid and gas that
#[derive(Debug, Clone, TypedBuilder, getset::CopyGetters, Serialize, Deserialize)]
pub struct Storage {
    /// Cargo storage provided
    #[getset(get_copy = "pub")]
    cargo: units::CargoSize,
    /// Liquid storage provided
    #[getset(get_copy = "pub")]
    liquid: units::LiquidVolume,
    /// Gas storage provided
    #[getset(get_copy = "pub")]
    gas: units::GasVolume,
}

/// Extra features of a building (in addition to reactions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtraFeature {
    /// The building is a core and must not be destroyed.
    Core,
    /// The building provides housing capacity, and inhabitants can be assigned to it.
    ProvidesHousing(u32),
    /// The building provides driving force for vehicles on adjacent rails.
    RailTerminal(units::RailForce),
    /// The building provides pumping force for adjacent liquid pipes.
    LiquidPump(units::PipeForce),
    /// The building provides pumping force for gas diffusion in adjacent corridors.
    GasPump(units::FanForce),
    /// Inhabitants with low skill may not be permitted to enter the node.
    SecureEntry {
        /// The skill type to check.
        skill: skill::TypeId,
        /// The minimum skill level required to enter the building.
        min_level: units::Skill,
        /// The probability per second per inhabitant that
        /// the inhabitant has lower skill level than required
        /// but still can enter the building.
        breach_probability: f64,
    },
    /// Inhabitants with low skill may not be permitted to exit the node.
    SecureExit {
        /// The skill type to check.
        skill: skill::TypeId,
        /// The minimum skill level required to exit the building.
        min_level: units::Skill,
        /// The probability per second per operator that
        /// the operator has lower skill level than required
        /// but still can exit the building.
        breach_probability: f64,
    },
}

/// Identifies a building category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryId(pub usize);

/// A category of building.
#[derive(Debug, Clone, TypedBuilder, getset::Getters, Serialize, Deserialize)]
pub struct Category {
    /// Title of the building category.
    #[getset(get = "pub")]
    title: ArcStr,
    /// Description of the building category.
    #[getset(get = "pub")]
    description: ArcStr,
}
