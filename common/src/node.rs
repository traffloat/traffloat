//! Node management.
//!
//! A node is an instance of a building.

use std::collections::BTreeMap;
use std::num::NonZeroUsize;

use arcstr::ArcStr;
use derive_new::new;
use legion::{systems::CommandBuffer, Entity};
use serde::{Deserialize, Serialize};
use smallvec::smallvec;
use typed_builder::TypedBuilder;

use crate::def::{building, GameDefinition};
use crate::defense;
use crate::population;
use crate::shape::{self, Shape};
use crate::space::{Matrix, Position};
use crate::sun::LightStats;
use crate::units;
use crate::SetupEcs;
use crate::{cargo, gas, liquid};

/// Component storing an identifier for a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, new, Serialize, Deserialize)]
pub struct Id {
    inner: u32,
}

/// Component storing the name of the node
#[derive(Debug, Clone, new, getset::Getters, getset::Setters, Serialize, Deserialize)]
pub struct Name {
    /// Name of the node
    #[getset(get = "pub")]
    #[getset(set = "pub")]
    name: ArcStr,
}

/// Indicates that a node is added
#[derive(Debug, new, getset::CopyGetters)]
pub struct AddEvent {
    /// The added node ID
    #[getset(get_copy = "pub")]
    node: Id,
    /// The added node entity
    #[getset(get_copy = "pub")]
    entity: Entity,
}

/// Indicates that a node is flagged for removal
#[derive(Debug, new, getset::CopyGetters)]
pub struct RemoveEvent {
    /// The removed node
    #[getset(get_copy = "pub")]
    node: Id,
}

/// Indicates that nodes have been removed
#[derive(Debug, new, getset::CopyGetters)]
pub struct PostRemoveEvent {
    /// Number of nodes removed
    #[getset(get_copy = "pub")]
    count: NonZeroUsize,
}

/// Tracks the nodes in the world
#[derive(Default)]
pub struct Index {
    index: BTreeMap<Id, Entity>,
    deletion_queue: Vec<Id>,
}

impl Index {
    /// Retrieves the entity ID for the given node
    pub fn get(&self, id: Id) -> Option<Entity> {
        self.index.get(&id).copied()
    }
}

#[codegen::system]
fn delete_nodes(
    cmd_buf: &mut legion::systems::CommandBuffer,
    #[resource] index: &mut Index,
    #[subscriber] removals: impl Iterator<Item = RemoveEvent>,
    #[publisher] post_remove_pub: impl FnMut(PostRemoveEvent),
) {
    for &node in &index.deletion_queue {
        let entity = index
            .index
            .remove(&node)
            .expect("Removing nonexistent node entity");
        cmd_buf.remove(entity);
    }
    let count = index.deletion_queue.len();
    index.deletion_queue.clear();
    if let Some(count) = NonZeroUsize::new(count) {
        post_remove_pub(PostRemoveEvent { count });
    }

    // queue deletion requests for the next event loop
    for removal in removals {
        index.deletion_queue.push(removal.node);
    }
}

/// An event to schedule requests to initialize new nodes.
///
/// Do not subscribe to this event for listening to node creation.
/// Use [`AddEvent`] instead.
#[derive(TypedBuilder)]
pub struct CreationRequest {
    /// Type ID of the building to create.
    type_id: building::TypeId,
    /// The position of the node.
    position: Position,
    /// The rotation matrix of the node.
    rotation: Matrix,
}

#[codegen::system]
fn create_new_node(
    entities: &mut CommandBuffer,
    #[subscriber] requests: impl Iterator<Item = CreationRequest>,
    #[publisher] add_events: impl FnMut(AddEvent),
    #[resource(no_init)] def: &GameDefinition,
    #[resource] index: &mut Index,
) {
    for request in requests {
        let building = def
            .building()
            .get(&request.type_id)
            .expect("Received invalid type ID");

        let id = Id::new(rand::random());

        let entity = entities.push((
            id,
            Name::new(building.name().clone()),
            request.position,
            Shape::builder()
                .unit(building.shape().unit())
                .matrix(request.rotation * building.shape().transform())
                .texture(shape::Texture::new(
                    building.shape().texture_src().clone(),
                    building.shape().texture_name().clone(),
                ))
                .build(),
            LightStats::default(),
            units::Portion::full(building.hitpoint()),
            cargo::StorageList::new(smallvec![]),
            cargo::StorageCapacity::new(building.storage().cargo()),
            liquid::StorageList::new(smallvec![]),
            liquid::StorageCapacity::new(building.storage().liquid()),
            gas::StorageList::new(smallvec![]),
            gas::StorageCapacity::new(building.storage().gas()),
        ));
        index.index.insert(id, entity);

        for feature in building.features() {
            match feature {
                building::ExtraFeature::Core => {
                    entities.add_component(entity, defense::Core);
                }
                building::ExtraFeature::ProvidesHousing(housing) => {
                    entities.add_component(
                        entity,
                        population::Housing::builder().capacity(*housing).build(),
                    );
                }
                _ => todo!(),
            }
        }

        add_events(AddEvent { node: id, entity })
    }
}

/// An event to schedule requests to initialize saved nodes.
///
/// Do not subscribe to this event for listening to node creation.
/// Use [`AddEvent`] instead.
#[derive(TypedBuilder)]
pub struct LoadRequest {
    /// The saved node.
    save: Box<save::Node>,
}

#[codegen::system]
fn create_saved_node(
    entities: &mut CommandBuffer,
    #[subscriber] requests: impl Iterator<Item = LoadRequest>,
    #[publisher] add_events: impl FnMut(AddEvent),
    #[resource] index: &mut Index,
) {
    for LoadRequest { save } in requests {
        let cargo_list = save
            .cargo
            .iter()
            .map(|(id, &size)| {
                let entity = entities.push((
                    cargo::Storage::new(id.clone()),
                    cargo::StorageSize::new(size),
                    cargo::NextStorageSize::new(size),
                ));
                (id.clone(), entity)
            })
            .collect();
        let liquid_list = save
            .liquid
            .iter()
            .map(|(id, &size)| {
                let entity = entities.push((
                    liquid::Storage::new(id.clone()),
                    liquid::StorageSize::new(size),
                    liquid::NextStorageSize::new(size),
                ));
                (id.clone(), entity)
            })
            .collect();
        let gas_list = save
            .gas
            .iter()
            .map(|(id, &size)| {
                let entity = entities.push((
                    gas::Storage::new(id.clone()),
                    gas::StorageSize::new(size),
                    gas::NextStorageSize::new(size),
                ));
                (id.clone(), entity)
            })
            .collect();

        let entity = entities.push((
            save.id,
            save.name.clone(),
            save.position,
            save.shape.clone(),
            LightStats::default(),
            save.hitpoint,
            cargo::StorageList::new(cargo_list),
            cargo::StorageCapacity::new(save.cargo_capacity),
            liquid::StorageList::new(liquid_list),
            liquid::StorageCapacity::new(save.liquid_capacity),
            gas::StorageList::new(gas_list),
            gas::StorageCapacity::new(save.gas_capacity),
        ));
        index.index.insert(save.id, entity);
        add_events(AddEvent {
            node: save.id,
            entity,
        })
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup
        .uses(delete_nodes_setup)
        .uses(create_new_node_setup)
        .uses(create_saved_node_setup)
}

/// Save type for nodes.
pub mod save {
    use std::collections::BTreeMap;

    use super::*;
    use crate::def;
    use crate::units;

    /// Saves all data related to a node.
    #[derive(Clone, Serialize, Deserialize)]
    pub struct Node {
        pub(crate) id: super::Id,
        pub(crate) name: super::Name,
        pub(crate) position: Position,
        pub(crate) shape: Shape,
        pub(crate) hitpoint: units::Portion<units::Hitpoint>,
        pub(crate) cargo: BTreeMap<def::cargo::TypeId, units::CargoSize>,
        pub(crate) cargo_capacity: units::CargoSize,
        pub(crate) liquid: BTreeMap<def::liquid::TypeId, units::LiquidVolume>,
        pub(crate) liquid_capacity: units::LiquidVolume,
        pub(crate) gas: BTreeMap<def::gas::TypeId, units::GasVolume>,
        pub(crate) gas_capacity: units::GasVolume,
    }
}
