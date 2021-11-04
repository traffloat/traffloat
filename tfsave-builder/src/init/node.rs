use std::collections::BTreeMap;

use codegen::{Definition, IdStr, ResolveContext};
use serde::{Deserialize, Serialize};
use traffloat_def::{building, cargo, gas, liquid};
use traffloat_types::space::{Position, TransformMatrix};
use traffloat_types::units;

#[derive(Clone, Serialize, Deserialize, Definition)]
#[hf_always]
#[hf_post_convert(post_node_convert)]
#[reuse_context(building => building::storage::liquid::Id)]
pub struct Def {
    pub(super) id:       Id,
    pub(super) id_str:   IdStr,
    pub(super) building: building::Id,
    pub(super) position: Position,
    #[hf_serde(default)]
    pub(super) rotation: TransformMatrix,
    pub(super) hitpoint: units::Hitpoint,

    #[hf_serde(default)]
    pub(super) cargo:  Vec<CargoEntry>,
    #[hf_serde(default)]
    pub(super) gas:    Vec<GasEntry>,
    #[hf_serde(default)]
    pub(super) liquid: Vec<LiquidEntry>,
}

/// Hack type to let [`super::edge`] know the building type of a node.
#[derive(Default)]
pub(super) struct NodeBuildingMap(pub(super) BTreeMap<Id, building::Id>);

fn post_node_convert(node: &mut Def, context: &mut ResolveContext) -> anyhow::Result<()> {
    let mut map = context.get_other::<NodeBuildingMap>();
    map.0.insert(node.id, node.building);
    Ok(())
}

#[derive(Clone, Serialize, Deserialize, Definition)]
#[hf_always]
pub struct CargoEntry {
    pub(super) ty:   cargo::Id,
    pub(super) size: units::CargoSize,
}

#[derive(Clone, Serialize, Deserialize, Definition)]
#[hf_always]
pub struct GasEntry {
    pub(super) ty:     gas::Id,
    pub(super) volume: units::GasVolume,
}

#[derive(Clone, Serialize, Deserialize, Definition)]
#[hf_always]
pub struct LiquidEntry {
    pub(super) storage: building::storage::liquid::Id,
    pub(super) ty:      liquid::Id,
    pub(super) volume:  units::LiquidVolume,
}
