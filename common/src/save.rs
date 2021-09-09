//! Saving game definition and state.

use std::convert::TryInto;

use cfg_if::cfg_if;
use legion::world::SubWorld;
use legion::{world::ComponentError, Entity, EntityStore, IntoQuery};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::clock::Clock;
use crate::def::GameDefinition;
use crate::defense;
use crate::edge::save::{Edge, SavedDuct};
use crate::node::save::Node;
use crate::population;
use crate::shape::Shape;
use crate::space::Position;
use crate::time::Instant;
use crate::units;
use crate::SetupEcs;
use crate::{cargo, gas, liquid, vehicle};
use crate::{edge, node};
use safety::Safety;

/// The save schema version.
///
/// This value is only bumped when necessary to distinguish incompatible formats.
pub const SCHEMA_VERSION: u32 = 0;

const TEXT_PREFIX: &str = "### SCHEMA_VERSION=";

/// The schema for a `.tsvt`/`.tsvb` file.
#[derive(getset::Getters, Serialize, Deserialize)]
pub struct SaveFile {
    /// Defines the game rules and mechanisms.
    #[getset(get = "pub")]
    def: GameDefinition,
    /// Defines the current state of the game.
    #[getset(get = "pub")]
    state: GameState,
}

/// The state of the game.
#[derive(Default, Serialize, Deserialize)]
pub struct GameState {
    // we need to box Node and Edge because they will be passed as boxed later on.
    #[allow(clippy::vec_box)]
    nodes: Vec<Box<Node>>,
    #[allow(clippy::vec_box)]
    edges: Vec<Box<Edge>>,
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
    #[cfg(feature = "save-text")]
    Text,
    /// Binary-based save format.
    ///
    /// Currently, this format uses [`rmp-serde`] as backend.
    #[cfg(feature = "save-binary")]
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
#[read_component(liquid::Storage)]
#[read_component(liquid::StorageList)]
#[read_component(liquid::StorageCapacity)]
#[read_component(liquid::StorageSize)]
#[read_component(gas::StorageList)]
#[read_component(gas::StorageCapacity)]
#[read_component(gas::StorageSize)]
#[read_component(defense::Core)]
#[read_component(population::Housing)]
#[read_component(vehicle::RailPump)]
#[read_component(liquid::Pump)]
#[read_component(gas::Pump)]
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
            state:
                GameState {
                    nodes: <(
                        Entity,
                        &node::Id,
                        &node::Name,
                        &Position,
                        &Shape,
                        &units::Portion<units::Hitpoint>,
                        &cargo::StorageList,
                        &cargo::StorageCapacity,
                        &liquid::StorageList,
                        &gas::StorageList,
                        &gas::StorageCapacity,
                    )>::query()
                    .iter(world)
                    .map(
                        |(
                            entity,
                            &id,
                            name,
                            &position,
                            shape,
                            &hitpoint,
                            cargo,
                            &cargo_capacity,
                            liquid,
                            gas,
                            &gas_capacity,
                        )| {
                            let entry =
                                world.entry_ref(*entity).expect("entity from query does not exist");
                            Box::new(Node {
                                id,
                                name: name.clone(),
                                position,
                                shape: shape.clone(),
                                hitpoint,
                                cargo: cargo
                                    .storages()
                                    .iter()
                                    .map(|(id, entity)| {
                                        let entry = world
                                            .entry_ref(*entity)
                                            .expect("Cargo storage entity is nonexistent");
                                        let size: &cargo::StorageSize = entry
                                            .get_component()
                                            .expect("Cargo storage entity has no StorageSize");
                                        (id.clone(), size.size())
                                    })
                                    .collect(),
                                cargo_capacity: cargo_capacity.total(),
                                liquid: liquid
                                    .storages()
                                    .iter()
                                    .map(|entity| {
                                        let entry = world
                                            .entry_ref(*entity)
                                            .expect("liquid storage entity is nonexistent");
                                        let storage: &liquid::Storage = entry
                                            .get_component()
                                            .expect("liquid storage entity has no Storage");
                                        let capacity: &liquid::StorageCapacity = entry
                                            .get_component()
                                            .expect("liquid storage entity has no StorageCapacity");
                                        let size: &liquid::StorageSize = entry
                                            .get_component()
                                            .expect("liquid storage entity has no StorageSize");
                                        node::save::LiquidStorage {
                                            ty: storage.liquid().clone(),
                                            capacity: capacity.total(),
                                            volume: size.size(),
                                        }
                                    })
                                    .collect(),
                                gas: gas
                                    .storages()
                                    .iter()
                                    .map(|(id, entity)| {
                                        let entry = world
                                            .entry_ref(*entity)
                                            .expect("gas storage entity is nonexistent");
                                        let size: &gas::StorageSize = entry
                                            .get_component()
                                            .expect("gas storage entity has no StorageSize");
                                        (id.clone(), size.size())
                                    })
                                    .collect(),
                                gas_capacity: gas_capacity.total(),
                                is_core: match entry.get_component::<defense::Core>() {
                                    Ok(_) => true,
                                    Err(ComponentError::NotFound { .. }) => false,
                                    Err(err) => panic!("{:?}", err),
                                },
                                housing_provision: match entry
                                    .get_component::<population::Housing>()
                                {
                                    Ok(housing) => Some(housing.capacity()),
                                    Err(ComponentError::NotFound { .. }) => None,
                                    Err(err) => panic!("{:?}", err),
                                },
                                rail_pump: match entry.get_component::<vehicle::RailPump>() {
                                    Ok(pump) => Some(pump.force()),
                                    Err(ComponentError::NotFound { .. }) => None,
                                    Err(err) => panic!("{:?}", err),
                                },
                                liquid_pump: match entry.get_component::<liquid::Pump>() {
                                    Ok(pump) => Some(pump.force()),
                                    Err(ComponentError::NotFound { .. }) => None,
                                    Err(err) => panic!("{:?}", err),
                                },
                                gas_pump: match entry.get_component::<gas::Pump>() {
                                    Ok(pump) => Some(pump.force()),
                                    Err(ComponentError::NotFound { .. }) => None,
                                    Err(err) => panic!("{:?}", err),
                                },
                            })
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
                        Box::new(Edge {
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
                        })
                    })
                    .collect(),
                    clock: clock.now(),
                },
        };

        match emit(&file, request) {
            Ok(data) => results(Response { data, format: request.format }),
            Err(err) => log::error!("Error saving game: {:?}", err),
        }
    }
}

/// Encodes a save file into a buffer.
pub fn emit(file: &SaveFile, request: &Request) -> anyhow::Result<Vec<u8>> {
    use anyhow::Context;

    Ok(match request.format {
        #[cfg(feature = "save-text")]
        Format::Text => {
            let string = format!("{}{:08X}\n", TEXT_PREFIX, SCHEMA_VERSION);

            let mut vec = string.into_bytes();
            serde_yaml::to_writer(&mut vec, file)
                .context("Save data are not compatible with YAML")?;

            vec
        }
        #[cfg(feature = "save-binary")]
        Format::Binary => {
            let mut vec = vec![0xFF_u8];
            vec.extend(SCHEMA_VERSION.to_le_bytes());

            let mut write =
                flate2::write::DeflateEncoder::new(&mut vec, flate2::Compression::best());
            rmp_serde::encode::write_named(&mut write, file)
                .context("Save data are not compatible with MessagePack")?;
            write.finish().context("Cannot compress MessagePack data")?;

            vec
        }
    })
}

/// Parses the buffer into a save file.
pub fn parse(mut buf: &[u8]) -> anyhow::Result<SaveFile> {
    use anyhow::Context;

    let (format, schema_version) = if buf.get(0) == Some(&0xFF) {
        cfg_if! {
            if #[cfg(feature = "save-binary")] {
                let format = Format::Binary;
            } else {
                anyhow::bail!("Not compiled with binary save support");
            }
        };

        let bytes = buf.get(1..5).context("Checking schema version")?;
        buf = buf.get(5..).expect("Checked in the previous line");
        let version = u32::from_ne_bytes(bytes.try_into().expect("5 - 1 == 4"));
        (format, version)
    } else {
        cfg_if! {
            if #[cfg(feature = "save-text")] {
                let format = Format::Text;

                buf = buf
                    .strip_prefix(TEXT_PREFIX.as_bytes())
                    .context("Schema version is missing")?;
                let version = buf.get(0..8).context("Schema version underflows")?;
                anyhow::ensure!(buf.get(8) == Some(&b'\n'), "Schema version is malformed");
                buf = buf.get(9..).expect("Checked in the previous line");
                let version = std::str::from_utf8(version).context("Schema version is malformed")?;
                let version = u32::from_str_radix(version, 16).context("Schema version is malformed")?;
                (format, version)
            } else {
                anyhow::bail!("Not compiled with text save support")
            }
        }
    };

    if schema_version != SCHEMA_VERSION {
        anyhow::bail!("Update your client.");
    }

    let file: SaveFile = match format {
        #[cfg(feature = "save-text")]
        Format::Text => serde_yaml::from_slice(buf).context("Save format error"),
        #[cfg(feature = "save-binary")]
        Format::Binary => rmp_serde::from_read(flate2::read::DeflateDecoder::new(buf))
            .context("Save format error"),
    }?;

    Ok(file)
}

/// Loads a save file.
pub fn load(mut setup: SetupEcs, buf: &[u8], now: u64) -> anyhow::Result<SetupEcs> {
    let file = parse(buf)?;

    setup.resources.insert(file.def);
    setup.resources.get_mut_or_default::<Clock>().reset_time(file.state.clock, now.homosign());

    for node in file.state.nodes {
        setup = setup.publish_event(node::LoadRequest::builder().save(node).build());
    }

    for edge in file.state.edges {
        setup = setup.publish_event(edge::LoadRequest::builder().save(edge).build());
    }

    Ok(setup)
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(save_setup)
}
