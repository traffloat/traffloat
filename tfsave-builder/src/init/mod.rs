use std::convert::TryInto;

use anyhow::Context;
use codegen::Definition;
use serde::{Deserialize, Serialize};
use traffloat_def::state::{self, State};
use traffloat_def::{building, CustomizableName, Def as AnyDef};
use traffloat_types::time;

pub mod edge;
pub mod node;

#[derive(Clone, Deserialize)]
pub struct ScalarState {
    pub time: time::Instant,
}

#[derive(Clone, Serialize, Deserialize, Definition)]
#[hf_always]
#[serde(tag = "type")]
pub enum Init {
    Node(node::Def),
    Edge(edge::Def),
}

pub fn resolve_states(
    defs: &[AnyDef],
    init: &[Init],
    scalar: &ScalarState,
) -> anyhow::Result<State> {
    let mut state = State::new(scalar.time);

    for (i, init) in init.iter().enumerate() {
        match init {
            Init::Node(node) => {
                let node_id = i.try_into().expect("too many nodes");
                let building: &building::Def = defs
                    .iter()
                    .find_map(|def| match def {
                        AnyDef::Building(building) if building.id() == node.building => {
                            Some(building)
                        }
                        _ => None,
                    })
                    .context("Edge references nonexistent building")?;

                state.nodes_mut().push(
                    state::Node::builder()
                        .id(state::NodeId::new(node_id))
                        .building(node.building)
                        .name(CustomizableName::Original(building.name().clone()))
                        .position(node.position)
                        .rotation(node.rotation.0)
                        .hitpoint(node.hitpoint)
                        .cargo(
                            node.cargo
                                .iter()
                                .map(|entry| state::CargoStorageEntry::new(entry.ty, entry.size))
                                .collect(),
                        )
                        .gas(
                            node.gas
                                .iter()
                                .map(|entry| state::GasStorageEntry::new(entry.ty, entry.volume))
                                .collect(),
                        )
                        .liquid({
                            let mut entries = vec![
                                state::LiquidStorageEntry::default();
                                building.storage().liquid().len()
                            ];
                            for storage in &node.liquid {
                                let entry = entries
                                    .get_mut(storage.storage.as_index())
                                    .context("invalid liquid storage")?;
                                *entry = state::LiquidStorageEntry::new(storage.ty, storage.volume);
                            }
                            entries
                        })
                        .build(),
                )
            }
            Init::Edge(edge) => {
                let design = edge
                    .ducts
                    .iter()
                    .map(|duct| {
                        state::Duct::builder()
                            .center(duct.center)
                            .radius(duct.radius)
                            .ty(match duct.ty {
                                edge::DuctType::Rail { dir } => state::DuctType::Rail(match dir {
                                    edge::MaybeDirection::From2To => {
                                        Some(state::Direction::From2To)
                                    }
                                    edge::MaybeDirection::To2From => {
                                        Some(state::Direction::To2From)
                                    }
                                    edge::MaybeDirection::Disabled => None,
                                }),
                                edge::DuctType::Electricity { disabled } => {
                                    state::DuctType::Electricity(!disabled)
                                }
                                edge::DuctType::Liquid(ty) => state::DuctType::Liquid {
                                    dir:          match ty {
                                        edge::LiquidDuctType::From2To { .. } => {
                                            Some(state::Direction::From2To)
                                        }
                                        edge::LiquidDuctType::To2From { .. } => {
                                            Some(state::Direction::To2From)
                                        }
                                        edge::LiquidDuctType::Disabled => None,
                                    },
                                    src_storage:  match ty {
                                        edge::LiquidDuctType::From2To { src_storage, .. }
                                        | edge::LiquidDuctType::To2From { src_storage, .. } => {
                                            src_storage.as_index()
                                        }
                                        edge::LiquidDuctType::Disabled => 0,
                                    },
                                    dest_storage: match ty {
                                        edge::LiquidDuctType::From2To { dest_storage, .. }
                                        | edge::LiquidDuctType::To2From { dest_storage, .. } => {
                                            dest_storage.as_index()
                                        }
                                        edge::LiquidDuctType::Disabled => 0,
                                    },
                                },
                            })
                            .build()
                    })
                    .collect();

                state.edges_mut().push(
                    state::Edge::builder()
                        .from(state::NodeId::new(
                            edge.from.as_index().try_into().expect("Too many nodes"),
                        )) // toml position = persistent ID
                        .to(state::NodeId::new(
                            edge.to.as_index().try_into().expect("Too many nodes"),
                        )) // toml position = persistent ID
                        .radius(edge.radius)
                        .hitpoint(edge.hitpoint)
                        .design(design)
                        .build(),
                )
            }
        }
    }

    Ok(state)
}
