//! Node management.
//!
//! A node is an instance of a building.

use std::collections::BTreeMap;
use std::num::NonZeroUsize;

use arcstr::ArcStr;
use derive_new::new;
use legion::Entity;
use serde::{Deserialize, Serialize};
use smallvec::smallvec;

use crate::def::{building, GameDefinition};
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
    /// The added node
    #[getset(get_copy = "pub")]
    node: Id,
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

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(delete_nodes_setup)
}

/// Initialize a new node entity.
///
/// Note that the caller should trigger [`AddEvent`] separately.
pub fn create_components(
    world: &mut impl legion::PushEntity,
    index: &mut Index,
    def: &GameDefinition,
    type_id: building::TypeId,
    position: Position,
    rotation: Matrix,
) -> Entity {
    let building = def.get_building(type_id);

    let id = Id::new(rand::random());

    let entity = world.push((
        id,
        Name::new(building.name().clone()),
        position,
        Shape::builder()
            .unit(building.shape().unit())
            .matrix(rotation * building.shape().transform())
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
    entity
}

/// Creates the components for a saved node.
///
/// Note that the caller should trigger [`AddEvent`] separately.
pub fn create_components_from_save(
    world: &mut impl legion::PushEntity,
    index: &mut Index,
    save: save::Node,
) -> Entity {
    let cargo_list = save
        .cargo
        .iter()
        .map(|(&id, &size)| {
            let entity = world.push((
                cargo::Storage::new(id),
                cargo::StorageSize::new(size),
                cargo::NextStorageSize::new(size),
            ));
            (id, entity)
        })
        .collect();
    let liquid_list = save
        .liquid
        .iter()
        .map(|(&id, &size)| {
            let entity = world.push((
                liquid::Storage::new(id),
                liquid::StorageSize::new(size),
                liquid::NextStorageSize::new(size),
            ));
            (id, entity)
        })
        .collect();
    let gas_list = save
        .gas
        .iter()
        .map(|(&id, &size)| {
            let entity = world.push((
                gas::Storage::new(id),
                gas::StorageSize::new(size),
                gas::NextStorageSize::new(size),
            ));
            (id, entity)
        })
        .collect();

    let entity = world.push((
        save.id,
        save.name.clone(),
        save.position,
        save.shape,
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
    entity
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
