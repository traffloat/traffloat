use derive_new::new;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use traffloat_types::space::{Matrix, Position};
use traffloat_types::units;
use typed_builder::TypedBuilder;

use crate::state::appearance::Appearance;
use crate::{building, cargo, gas, liquid, CustomizableName};

/// Component storing a persistent identifier for a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, new, Serialize, Deserialize)]
pub struct NodeId {
    inner: u32,
}

// This should not belong to the def crate, but there is no better way to deal with E0117.
codegen::component_depends! {
    NodeId = (
        NodeId,
        building::Id,
        CustomizableName,
        Position,
        Appearance,
        units::Portion<units::Hitpoint>,
    ) + ?()
}

/// The state of a node.
///
/// State of population storages are stored in the inhabitant states.
#[derive(Getters, CopyGetters, TypedBuilder, Serialize, Deserialize)]
pub struct Node {
    /// Persistent ID of the node.
    #[getset(get_copy = "pub")]
    id:       NodeId,
    /// Building type of the node.
    #[getset(get_copy = "pub")]
    building: building::Id,
    /// Name of the node.
    #[getset(get = "pub")]
    name:     CustomizableName,
    /// Position of the node.
    #[getset(get_copy = "pub")]
    position: Position,
    /// Rotation of the node.
    #[getset(get_copy = "pub")]
    rotation: Matrix,
    /// Hitpoint of the node.
    #[getset(get_copy = "pub")]
    hitpoint: units::Hitpoint,
    /// State of cargo storage in the node.
    #[getset(get = "pub")]
    cargo:    Vec<CargoStorageEntry>,
    /// State of gas storage in the node.
    #[getset(get = "pub")]
    gas:      Vec<GasStorageEntry>,
    /// State of liquid storages in the node.
    #[getset(get = "pub")]
    liquid:   Vec<LiquidStorageEntry>,
}

#[derive(Debug, Clone, Copy, CopyGetters, new, Serialize, Deserialize)]
pub struct CargoStorageEntry {
    #[getset(get_copy = "pub")]
    cargo: cargo::Id,
    #[getset(get_copy = "pub")]
    size:  units::CargoSize,
}

#[derive(Debug, Clone, Copy, CopyGetters, new, Serialize, Deserialize)]
pub struct GasStorageEntry {
    #[getset(get_copy = "pub")]
    gas:    gas::Id,
    #[getset(get_copy = "pub")]
    volume: units::GasVolume,
}

#[derive(Debug, Clone, Copy, Default, CopyGetters, new, Serialize, Deserialize)]
pub struct LiquidStorageEntry {
    #[getset(get_copy = "pub")]
    liquid: liquid::Id,
    #[getset(get_copy = "pub")]
    volume: units::LiquidVolume,
}
