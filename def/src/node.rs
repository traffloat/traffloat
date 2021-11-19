//! Node states.

use derive_new::new;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use traffloat_types::space::{Matrix, Position};
use traffloat_types::units;
use typed_builder::TypedBuilder;
use xylem::Identifiable;

use crate::{building, cargo, gas, liquid, CustomizableName, Schema};

/// Component storing a persistent identifier for a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, new, Serialize, Deserialize)]
pub struct NodeId {
    inner: u32,
}

#[cfg(feature = "xy")]
impl Identifiable<Schema> for Node {
    type Scope = ();

    fn id(&self) -> crate::Id<Node> {
        crate::Id::new(self.id.inner.try_into().expect("Too many items"))
    }
}

// This should not belong to the def crate, but there is no better way to deal with E0117.
codegen::component_depends! {
    NodeId = (
        NodeId,
        building::Id,
        CustomizableName,
        Position,
        units::Portion<units::Hitpoint>,
    ) + ?()
}

/// The state of a node.
///
/// State of population storages are stored in the inhabitant states.
#[derive(Debug, Clone, Getters, CopyGetters, TypedBuilder, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize), process))]
pub struct Node {
    /// Persistent ID of the node.
    #[getset(get_copy = "pub")]
    #[cfg_attr(feature = "xy", xylem(args(new = true)))]
    id:       NodeId,
    /// Building type of the node.
    #[getset(get_copy = "pub")]
    #[cfg_attr(feature = "xy", xylem(args(import = vec![std::any::TypeId::of::<building::storage::liquid::Def>()])))]
    building: building::Id,
    /// Name of the node.
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(default = CustomizableName::Custom(arcstr::literal!(""))))]
    name:     CustomizableName,
    /// Position of the node.
    #[getset(get_copy = "pub")]
    position: Position,
    /// Rotation of the node.
    #[getset(get_copy = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    rotation: Matrix,
    /// Hitpoint of the node.
    #[getset(get_copy = "pub")]
    hitpoint: units::Hitpoint,
    /// State of cargo storage in the node.
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    cargo:    Vec<CargoStorageEntry>,
    /// State of gas storage in the node.
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    gas:      Vec<GasStorageEntry>,
    /// State of liquid storages in the node.
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    liquid:   Vec<LiquidStorageEntry>,
}

/// A cargo entry storage, representing the size of a cargo type.
#[derive(Debug, Clone, Copy, CopyGetters, new, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct CargoStorageEntry {
    /// The cargo type.
    #[getset(get_copy = "pub")]
    ty:   cargo::Id,
    /// The cargo size.
    #[getset(get_copy = "pub")]
    size: units::CargoSize,
}

/// A gas entry storage, representing the volume of a gas type.
#[derive(Debug, Clone, Copy, CopyGetters, new, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct GasStorageEntry {
    /// The gas type.
    #[getset(get_copy = "pub")]
    ty:     gas::Id,
    /// The gas volume.
    #[getset(get_copy = "pub")]
    volume: units::GasVolume,
}

/// A liquid entry storage, representing the volume of a liquid type.
#[derive(Debug, Clone, Copy, Default, CopyGetters, new, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct LiquidStorageEntry {
    /// The liquid type.
    #[getset(get_copy = "pub")]
    ty:     liquid::Id,
    /// The liquid volume.
    #[getset(get_copy = "pub")]
    volume: units::LiquidVolume,
}

/// Xylem-specific objects.
#[cfg(feature = "xy")]
pub mod xy {
    use std::any::TypeId;
    use std::collections::BTreeMap;

    use xylem::{Context, DefaultContext, IdArgs, Processable, Xylem};

    use super::{Node, NodeId};
    use crate::building::xy::BuildingNameMap;
    use crate::{building, CustomizableName, Schema};

    impl Xylem<Schema> for NodeId {
        type From = String;
        type Args = IdArgs;

        fn convert_impl(
            from: Self::From,
            context: &mut DefaultContext,
            args: &Self::Args,
        ) -> anyhow::Result<Self> {
            let id = crate::Id::<Node>::convert(from, context, args)?;

            Ok(NodeId { inner: id.index().try_into().expect("Too many items") })
        }
    }

    /// A mapping of node IDs to their building types.
    #[derive(Default)]
    pub struct NodeBuildingMap {
        map: BTreeMap<NodeId, building::Id>,
    }

    impl NodeBuildingMap {
        /// Insert a node ID.
        pub fn insert(&mut self, id: NodeId, item: building::Id) { self.map.insert(id, item); }

        /// Lookup a node ID.
        pub fn get(&self, id: NodeId) -> Option<building::Id> { self.map.get(&id).copied() }
    }

    impl Processable<Schema> for Node {
        fn postprocess(&mut self, context: &mut DefaultContext) -> anyhow::Result<()> {
            {
                let map =
                    context.get_mut::<BuildingNameMap, _>(TypeId::of::<()>(), Default::default);
                let item = map.get(self.building).expect("Dangling building reference");
                self.name = CustomizableName::Original(item.clone());
            }

            {
                let map =
                    context.get_mut::<NodeBuildingMap, _>(TypeId::of::<()>(), Default::default);
                map.insert(self.id, self.building);
            }

            Ok(())
        }
    }
}
