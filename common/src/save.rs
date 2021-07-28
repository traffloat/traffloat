//! Saving game definition and state.

use legion::world::SubWorld;
use legion::EntityStore;
use legion::IntoQuery;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::clock::Clock;
use crate::def::GameDefinition;
use crate::edge::save::{Edge, SavedDuct};
use crate::node::save::Node;
use crate::shape::Shape;
use crate::space::Position;
use crate::time::Instant;
use crate::units;
use crate::SetupEcs;
use crate::{cargo, gas, liquid};
use crate::{edge, node};

/// The save schema version.
///
/// This value is only bumped when necessary to distinguish incompatible formats.
pub const SCHEMA_VERSION: u32 = 0;

/// The schema for a `.tsvt`/`.tsvb` file.
#[derive(Serialize, Deserialize)]
pub struct SaveFile {
    def: GameDefinition,
    state: GameState,
}

/// The state of the game.
#[derive(Serialize, Deserialize)]
pub struct GameState {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    clock: Instant,
}

/// Requests saving game state.
#[derive(TypedBuilder, getset::CopyGetters)]
pub struct Request {
    /// The format to save as.
    #[getset(get_copy = "pub")]
    format: Format,
}

/// The format to save as.
#[derive(Debug, Clone, Copy)]
pub enum Format {
    /// Text-based save format.
    ///
    /// Currently, this format uses [`serde_yaml`] as backend.
    #[cfg(feature = "serde_yaml")]
    Text,
    /// Binary-based save format.
    ///
    /// Currently, this format uses [`rmp-serde`] as backend.
    #[cfg(feature = "rmp-serde")]
    Binary,
}

/// Result for saving game state.
#[derive(getset::Getters, getset::CopyGetters)]
pub struct Response {
    /// The format of this response.
    #[getset(get_copy = "pub")]
    format: Format,
    /// The raw result data.
    #[getset(get = "pub")]
    data: Vec<u8>,
}

#[codegen::system]
#[read_component(node::Id)]
#[read_component(node::Name)]
#[read_component(edge::Id)]
#[read_component(edge::Size)]
#[read_component(edge::Design)]
#[read_component(Position)]
#[read_component(Shape)]
#[read_component(units::Portion<units::Hitpoint>)]
#[read_component(cargo::StorageList)]
#[read_component(cargo::StorageCapacity)]
#[read_component(cargo::StorageSize)]
#[read_component(liquid::StorageList)]
#[read_component(liquid::StorageCapacity)]
#[read_component(liquid::StorageSize)]
#[read_component(gas::StorageList)]
#[read_component(gas::StorageCapacity)]
#[read_component(gas::StorageSize)]
fn save(
    world: &mut SubWorld,
    #[subscriber] requests: impl Iterator<Item = Request>,
    #[publisher] results: impl FnMut(Response),
    #[resource] clock: &Clock,
    #[resource(no_init)] def: &GameDefinition,
) {
    for request in requests {
        let file: SaveFile = SaveFile {
            def: def.clone(),
            state: GameState {
                nodes: <(
                    &node::Id,
                    &node::Name,
                    &Position,
                    &Shape,
                    &units::Portion<units::Hitpoint>,
                    &cargo::StorageList,
                    &cargo::StorageCapacity,
                    &liquid::StorageList,
                    &liquid::StorageCapacity,
                    &gas::StorageList,
                    &gas::StorageCapacity,
                )>::query()
                .iter(world)
                .map(
                    |(
                        &id,
                        name,
                        &position,
                        shape,
                        &hitpoints,
                        cargo,
                        &cargo_capacity,
                        liquid,
                        &liquid_capacity,
                        gas,
                        &gas_capacity,
                    )| {
                        Node {
                            id,
                            name: name.clone(),
                            position,
                            shape: shape.clone(),
                            hitpoints,
                            cargo: cargo
                                .storages()
                                .iter()
                                .map(|&(id, entity)| {
                                    let entry = world
                                        .entry_ref(entity)
                                        .expect("Cargo storage entity is nonexistent");
                                    let size: &cargo::StorageSize = entry
                                        .get_component()
                                        .expect("Cargo storage entity has no StorageSize");
                                    (id, size.size())
                                })
                                .collect(),
                            cargo_capacity: cargo_capacity.total(),
                            liquid: liquid
                                .storages()
                                .iter()
                                .map(|&(id, entity)| {
                                    let entry = world
                                        .entry_ref(entity)
                                        .expect("liquid storage entity is nonexistent");
                                    let size: &liquid::StorageSize = entry
                                        .get_component()
                                        .expect("liquid storage entity has no StorageSize");
                                    (id, size.size())
                                })
                                .collect(),
                            liquid_capacity: liquid_capacity.total(),
                            gas: gas
                                .storages()
                                .iter()
                                .map(|&(id, entity)| {
                                    let entry = world
                                        .entry_ref(entity)
                                        .expect("gas storage entity is nonexistent");
                                    let size: &gas::StorageSize = entry
                                        .get_component()
                                        .expect("gas storage entity has no StorageSize");
                                    (id, size.size())
                                })
                                .collect(),
                            gas_capacity: gas_capacity.total(),
                        }
                    },
                )
                .collect(),
                edges: <(
                    &edge::Id,
                    &edge::Size,
                    &edge::Design,
                    &units::Portion<units::Hitpoint>,
                )>::query()
                .iter(world)
                .map(|(&id, &size, design, &hitpoint)| {
                    let &from = world
                        .entry_ref(id.from())
                        .expect("Edge points to nonexistent ID")
                        .get_component::<node::Id>()
                        .expect("Edge points to non-Node");
                    let &to = world
                        .entry_ref(id.to())
                        .expect("Edge points to nonexistent ID")
                        .get_component::<node::Id>()
                        .expect("Edge points to non-Node");
                    Edge {
                        from,
                        to,
                        size,
                        design: design
                            .ducts()
                            .iter()
                            .map(|duct| SavedDuct {
                                center: duct.center(),
                                radius: duct.radius(),
                                ty: duct.ty(),
                            })
                            .collect(),
                        hitpoint,
                    }
                })
                .collect(),
                clock: clock.now(),
            },
        };

        let ret = match request.format {
            #[cfg(feature = "serde_yaml")]
            Format::Text => serde_yaml::to_vec(&file).map_err(|err| err.to_string()),
            #[cfg(feature = "rmp-serde")]
            Format::Binary => rmp_serde::to_vec_named(&file).map_err(|err| err.to_string()),
        };

        match ret {
            Ok(mut data) => {
                match request.format {
                    Format::Text => {
                        let string = format!("### SCHEMA_VERSION={:08X}\n", SCHEMA_VERSION);
                        data.splice(0..0, string.bytes());
                    }
                    Format::Binary => {
                        let mut vec = arrayvec::ArrayVec::<u8, 5>::new();
                        vec.push(0xFF);
                        vec.extend(SCHEMA_VERSION.to_le_bytes());
                        data.splice(0..0, vec);
                    }
                }
                results(Response {
                    format: request.format,
                    data,
                });
            }
            Err(err) => {
                log::error!("Error saving game: {}", err);
            }
        }
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(save_setup)
}
