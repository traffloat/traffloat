//! Building definitions

use codegen::Definition;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use traffloat_types::space::TransformMatrix;
use traffloat_types::{geometry, units};

use crate::atlas::Sprite;
use crate::feature::Feature;
use crate::lang;

/// A type of building.
#[derive(Debug, Clone, CopyGetters, Getters, Serialize, Deserialize, Definition)]
pub struct Def {
    /// ID of the building type.
    #[getset(get_copy = "pub")]
    id:          Id,
    /// Name of the building type.
    #[getset(get = "pub")]
    name:        lang::Item,
    /// Short summary of the building type.
    #[getset(get = "pub")]
    summary:     lang::Item,
    /// Long description of the building type.
    #[getset(get = "pub")]
    description: lang::Item,
    /// Category of the building type.
    #[getset(get = "pub")]
    category:    category::Id,
    /// Shape of the building.
    ///
    /// If multiple shapes are provided, they are all rendered together in order.
    #[getset(get = "pub")]
    shapes:      Vec<Shape>,
    /// Maximum hitpoint of a building.
    ///
    /// The actual hitpoint is subject to asteroid and fire damage.
    /// It can be restored by construction work.
    #[getset(get_copy = "pub")]
    hitpoint:    units::Hitpoint,
    /// Storage provided by a building
    #[getset(get = "pub")]
    storage:     Storage,
    /// Extra features associated with the building.
    #[getset(get = "pub")]
    features:    Vec<Feature>,
}

/// Shape of a building.
#[derive(Debug, Clone, CopyGetters, Getters, Serialize, Deserialize, Definition)]
pub struct Shape {
    /// The unit model type.
    #[getset(get_copy = "pub")]
    unit:      geometry::Unit,
    /// The transformation matrix from the unit model to this shape.
    #[getset(get_copy = "pub")]
    transform: TransformMatrix,
    /// The texture of the building.
    #[getset(get = "pub")]
    texture:   Sprite,
}

/// Storage provided by a building.
///
/// This storage is also used as a buffer for liquid and gas transfer.
/// The storage size is the maximum total amount of liquid and gas that
/// pipe systems passing through this building can transfer per frame.
#[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize, Definition)]
pub struct Storage {
    /// Cargo storage provided
    #[getset(get_copy = "pub")]
    cargo:      units::CargoSize,
    /// Gas storage provided
    #[getset(get_copy = "pub")]
    gas:        units::GasVolume,
    /// Liquid storages provided
    #[getset(get = "pub")]
    liquid:     Vec<storage::liquid::Def>,
    /// Population storages provided
    #[getset(get = "pub")]
    population: Vec<storage::population::Def>,
}

/// Storages in buildings.
pub mod storage {
    /// Liquid storage.
    pub mod liquid {
        use codegen::Definition;
        use getset::{CopyGetters, Getters};
        use serde::{Deserialize, Serialize};
        use traffloat_types::units;

        use crate::lang;

        /// A liquid storage.
        ///
        /// A building can have multiple, inhomogeneous liquid storages,
        /// which can be individually addressed by their IDs.
        /// Reactions involving liquids can consume, store or use (for catalyst)
        /// specific liquid types from the named  storages.
        #[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize, Definition)]
        pub struct Def {
            /// ID of the liquid storage.
            #[getset(get_copy = "pub")]
            id:       Id,
            /// The capacity of this storage.
            #[getset(get_copy = "pub")]
            capacity: units::LiquidVolume,
            /// The name of this storage.
            #[getset(get = "pub")]
            name:     lang::Item,
        }
    }

    /// Population storage.
    pub mod population {
        use codegen::Definition;
        use getset::{CopyGetters, Getters};
        use serde::{Deserialize, Serialize};
        use traffloat_types::units;

        use crate::lang;

        /// A population storage, allowing inhabitants to temporarily stay in a node.
        ///
        /// All inhabitants entering a building by swimming or disembarking from a vehicle in the
        /// building would enter a population storage.
        #[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize, Definition)]
        pub struct Def {
            /// ID of the liquid storage.
            #[getset(get_copy = "pub")]
            id:       Id,
            /// The capacity of this storage.
            #[getset(get_copy = "pub")]
            capacity: units::LiquidVolume,
            /// The name of this storage.
            #[getset(get = "pub")]
            name:     lang::Item,
        }
    }

    // TODO vehicle storage, for moving vehicles from rail to rail.
}

/// Categories of buildings.
pub mod category {
    use codegen::Definition;
    use getset::Getters;
    use serde::{Deserialize, Serialize};

    use crate::lang;

    /// A category of building.
    #[derive(Debug, Clone, Getters, Serialize, Deserialize, Definition)]
    pub struct Def {
        /// ID of the building category.
        #[getset(get_copy = "pub")]
        id:          Id,
        /// Title of the building category.
        #[getset(get = "pub")]
        title:       lang::Item,
        /// Description of the building category.
        #[getset(get = "pub")]
        description: lang::Item,
    }
}
