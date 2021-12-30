//! Building definitions

use gusket::Gusket;
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
#[derive(Debug, Clone, Gusket, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize), process))]
#[gusket(all, immut)]
pub struct Def {
    /// ID of the building type.
    #[gusket(copy)]
    #[cfg_attr(feature = "xy", xylem(args(new = true)))]
    id:          Id,
    /// String ID of the building type.
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    id_str:      IdString<Def>,
    /// Name of the building type.
    name:        lang::Item,
    /// Short summary of the building type.
    summary:     lang::Item,
    /// Long description of the building type.
    description: lang::Item,
    /// Category of the building type.
    #[gusket(copy)]
    category:    category::Id,
    /// Shape of the building.
    ///
    /// If multiple shapes are provided, they are all rendered together in order.
    shapes:      Vec<Shape>,
    /// Maximum hitpoint of a building.
    ///
    /// The actual hitpoint is subject to asteroid and fire damage.
    /// It can be restored by construction work.
    #[gusket(copy)]
    hitpoint:    units::Hitpoint,
    /// Storage provided by a building
    storage:     Storage,
    /// Extra features associated with the building.
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
#[derive(Debug, Clone, Gusket, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
#[gusket(all, immut)]
pub struct Shape {
    /// The unit model type.
    #[gusket(copy)]
    unit:      geometry::Unit,
    /// The transformation matrix from the unit model to this shape.
    #[gusket(copy)]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    transform: Matrix,
    /// The texture of the building.
    #[gusket(copy)]
    texture:   ModelRef,
}

/// Storage provided by a building.
///
/// This storage is also used as a buffer for liquid and gas transfer.
/// The storage size is the maximum total amount of liquid and gas that
/// pipe systems passing through this building can transfer per frame.
#[derive(Debug, Clone, Gusket, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
#[gusket(all, immut)]
pub struct Storage {
    /// Cargo storage provided
    #[gusket(copy)]
    cargo:      units::CargoSize,
    /// Gas storage provided
    #[gusket(copy)]
    gas:        units::GasVolume,
    /// Liquid storages provided
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    liquid:     Vec<storage::liquid::Def>,
    /// Population storages provided
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    population: Vec<storage::population::Def>,
}

/// Storages in buildings.
pub mod storage {
    /// Liquid storage.
    pub mod liquid {
        use gusket::Gusket;
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
        #[derive(Debug, Clone, Gusket, Serialize, Deserialize)]
        #[cfg_attr(feature = "xy", derive(xylem::Xylem))]
        #[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
        #[gusket(all, immut)]
        pub struct Def {
            /// ID of the liquid storage.
            #[gusket(copy)]
            #[cfg_attr(feature = "xy", xylem(args(new = true, track = true)))]
            id:       Id,
            /// String ID of the liquid storage.
            #[cfg_attr(feature = "xy", xylem(serde(default)))]
            id_str:   IdString<Def>,
            /// The capacity of this storage.
            #[gusket(copy)]
            capacity: units::LiquidVolume,
            /// The name of this storage.
            name:     lang::Item,
        }
    }

    /// Population storage.
    pub mod population {
        use gusket::Gusket;
        use serde::{Deserialize, Serialize};

        use crate::{lang, IdString};

        /// Identifies a population storage.
        pub type Id = crate::Id<Def>;

        impl_identifiable!(Def, crate::building::Def);

        /// A population storage, allowing inhabitants to temporarily stay in a node.
        ///
        /// All inhabitants entering a building by swimming or disembarking from a vehicle in the
        /// building would enter a population storage.
        #[derive(Debug, Clone, Gusket, Serialize, Deserialize)]
        #[cfg_attr(feature = "xy", derive(xylem::Xylem))]
        #[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
        #[gusket(all, immut)]
        pub struct Def {
            /// ID of the population storage.
            #[gusket(copy)]
            #[cfg_attr(feature = "xy", xylem(args(new = true)))]
            id:       Id,
            /// String ID of the population storage.
            #[cfg_attr(feature = "xy", xylem(serde(default)))]
            id_str:   IdString<Def>,
            /// The capacity of this storage.
            #[gusket(copy)]
            capacity: u32,
            /// The name of this storage.
            name:     lang::Item,
        }
    }

    // TODO vehicle storage, for moving vehicles from rail to rail.
}

/// Categories of buildings.
pub mod category {
    use gusket::Gusket;
    use serde::{Deserialize, Serialize};

    use crate::{lang, IdString};

    /// Identifies a building category.
    pub type Id = crate::Id<Def>;

    impl_identifiable!(Def);

    /// A category of building.
    #[derive(Debug, Clone, Gusket, Serialize, Deserialize)]
    #[cfg_attr(feature = "xy", derive(xylem::Xylem))]
    #[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
    #[gusket(all, immut)]
    pub struct Def {
        /// ID of the building category.
        #[gusket(copy)]
        #[cfg_attr(feature = "xy", xylem(args(new = true)))]
        id:          Id,
        /// String ID of the building category.
        #[cfg_attr(feature = "xy", xylem(serde(default)))]
        id_str:      IdString<Def>,
        /// Title of the building category.
        title:       lang::Item,
        /// Description of the building category.
        description: lang::Item,
    }
}
