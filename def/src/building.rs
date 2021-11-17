//! Building definitions

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use traffloat_types::space::Matrix;
use traffloat_types::{geometry, units};

use crate::atlas::ModelRef;
use crate::feature::Feature;
use crate::{lang, IdString};

/// Identifies a building type.
pub type Id = crate::Id<Def>;

impl_identifiable!(Def);

/// A type of building.
#[derive(Debug, Clone, CopyGetters, Getters, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize), process))]
pub struct Def {
    /// ID of the building type.
    #[getset(get_copy = "pub")]
    #[cfg_attr(feature = "xy", xylem(args(new = true)))]
    id:          Id,
    /// String ID of the building type.
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    id_str:      IdString<Def>,
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
    #[getset(get_copy = "pub")]
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
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    features:    Vec<Feature>,
}

/// Xylem-specific objects.
#[cfg(feature = "xy")]
pub mod xy {
    use std::any::TypeId;
    use std::collections::BTreeMap;

    use xylem::{Context, DefaultContext, Processable};

    use super::Id;
    use crate::{lang, Schema};

    /// A mapping of building type IDs to their names.
    #[derive(Default)]
    pub struct BuildingNameMap {
        map: BTreeMap<Id, lang::Item>,
    }

    impl BuildingNameMap {
        /// Insert a building ID.
        pub fn insert(&mut self, id: Id, item: lang::Item) { self.map.insert(id, item); }

        /// Lookup a building ID.
        pub fn get(&self, id: Id) -> Option<&lang::Item> { self.map.get(&id) }
    }

    impl Processable<Schema> for super::Def {
        fn postprocess(&mut self, context: &mut DefaultContext) -> anyhow::Result<()> {
            let map = context.get_mut::<BuildingNameMap, _>(TypeId::of::<()>(), Default::default);
            map.insert(self.id, self.name.clone());
            Ok(())
        }
    }
}

/// Shape of a building.
#[derive(Debug, Clone, CopyGetters, Getters, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct Shape {
    /// The unit model type.
    #[getset(get_copy = "pub")]
    unit:      geometry::Unit,
    /// The transformation matrix from the unit model to this shape.
    #[getset(get_copy = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    transform: Matrix,
    /// The texture of the building.
    #[getset(get_copy = "pub")]
    texture:   ModelRef,
}

/// Storage provided by a building.
///
/// This storage is also used as a buffer for liquid and gas transfer.
/// The storage size is the maximum total amount of liquid and gas that
/// pipe systems passing through this building can transfer per frame.
#[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct Storage {
    /// Cargo storage provided
    #[getset(get_copy = "pub")]
    cargo:      units::CargoSize,
    /// Gas storage provided
    #[getset(get_copy = "pub")]
    gas:        units::GasVolume,
    /// Liquid storages provided
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    liquid:     Vec<storage::liquid::Def>,
    /// Population storages provided
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    population: Vec<storage::population::Def>,
}

/// Storages in buildings.
pub mod storage {
    /// Liquid storage.
    pub mod liquid {
        use getset::{CopyGetters, Getters};
        use serde::{Deserialize, Serialize};
        use traffloat_types::units;

        use crate::{lang, IdString};

        /// Identifies a liquid storage.
        pub type Id = crate::Id<Def>;

        impl_identifiable!(Def, crate::building::Def);

        /// A liquid storage.
        ///
        /// A building can have multiple, inhomogeneous liquid storages,
        /// which can be individually addressed by their IDs.
        /// Reactions involving liquids can consume, store or use (for catalyst)
        /// specific liquid types from the named  storages.
        #[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize)]
        #[cfg_attr(feature = "xy", derive(xylem::Xylem))]
        #[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
        pub struct Def {
            /// ID of the liquid storage.
            #[getset(get_copy = "pub")]
            #[cfg_attr(feature = "xy", xylem(args(new = true, track = true)))]
            id:       Id,
            /// String ID of the liquid storage.
            #[getset(get = "pub")]
            #[cfg_attr(feature = "xy", xylem(serde(default)))]
            id_str:   IdString<Def>,
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
        use getset::{CopyGetters, Getters};
        use serde::{Deserialize, Serialize};

        use crate::{lang, IdString};

        /// Identifies a population storage.
        pub type Id = crate::Id<Def>;

        impl_identifiable!(Def, crate::building::Def);

        /// A population storage, allowing inhabitants to temporarily stay in a node.
        ///
        /// All inhabitants entering a building by swimming or disembarking from a vehicle in the
        /// building would enter a population storage.
        #[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize)]
        #[cfg_attr(feature = "xy", derive(xylem::Xylem))]
        #[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
        pub struct Def {
            /// ID of the population storage.
            #[getset(get_copy = "pub")]
            #[cfg_attr(feature = "xy", xylem(args(new = true)))]
            id:       Id,
            /// String ID of the population storage.
            #[getset(get = "pub")]
            #[cfg_attr(feature = "xy", xylem(serde(default)))]
            id_str:   IdString<Def>,
            /// The capacity of this storage.
            #[getset(get_copy = "pub")]
            capacity: u32,
            /// The name of this storage.
            #[getset(get = "pub")]
            name:     lang::Item,
        }
    }

    // TODO vehicle storage, for moving vehicles from rail to rail.
}

/// Categories of buildings.
pub mod category {
    use getset::{CopyGetters, Getters};
    use serde::{Deserialize, Serialize};

    use crate::{lang, IdString};

    /// Identifies a building category.
    pub type Id = crate::Id<Def>;

    impl_identifiable!(Def);

    /// A category of building.
    #[derive(Debug, Clone, CopyGetters, Getters, Serialize, Deserialize)]
    #[cfg_attr(feature = "xy", derive(xylem::Xylem))]
    #[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
    pub struct Def {
        /// ID of the building category.
        #[getset(get_copy = "pub")]
        #[cfg_attr(feature = "xy", xylem(args(new = true)))]
        id:          Id,
        /// String ID of the building category.
        #[getset(get = "pub")]
        #[cfg_attr(feature = "xy", xylem(serde(default)))]
        id_str:      IdString<Def>,
        /// Title of the building category.
        #[getset(get = "pub")]
        title:       lang::Item,
        /// Description of the building category.
        #[getset(get = "pub")]
        description: lang::Item,
    }
}
