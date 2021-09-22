//! Building definitions

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use super::{feature::Feature, skill};
use crate::space::Matrix;
use crate::{geometry, units};

/// Identifies a building type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TypeId(pub ArcStr);

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
    #[getset(get = "pub")]
    category: CategoryId,
    /// Shape of the building.
    #[getset(get = "pub")]
    shape: Shape,
    /// Maximum hitpoint of a building.
    ///
    /// The actual hitpoint is subject to asteroid and fire damage.
    /// It can be restored by construction work.
    #[getset(get_copy = "pub")]
    hitpoint: units::Hitpoint,
    /// Storage provided by a building
    #[getset(get = "pub")]
    storage: Storage,
    /// Extra features associated with the building.
    #[getset(get = "pub")]
    features: Vec<Feature>,
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

/// Storage provided by a building.
///
/// This storage is also used as a buffer for liquid and gas transfer.
/// The storage size is the maximum amount of liquid and gas that
#[derive(
    Debug, Clone, TypedBuilder, getset::Getters, getset::CopyGetters, Serialize, Deserialize,
)]
pub struct Storage {
    /// Cargo storage provided
    #[getset(get_copy = "pub")]
    cargo: units::CargoSize,
    /// Liquid storage provided
    #[getset(get = "pub")]
    liquid: Vec<units::LiquidVolume>,
    /// Gas storage provided
    #[getset(get_copy = "pub")]
    gas: units::GasVolume,
}

/// Identifies a building category
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CategoryId(pub ArcStr);

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
