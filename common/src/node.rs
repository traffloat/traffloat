//! Node management.
//!
//! A node is an instance of a building.

use std::collections::BTreeMap;
use std::num::NonZeroUsize;

use arcstr::ArcStr;
use derive_new::new;
use legion::Entity;
use serde::{Deserialize, Serialize};

use crate::def::{building, GameDefinition};
use crate::shape::{self, Shape};
use crate::space::{Matrix, Position};
use crate::sun::LightStats;
use crate::SetupEcs;

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

/// Return type of [`create_components`].
pub type Components = (Id, Name, Position, Shape, LightStats);
/// Creates the components for a node entity.
pub fn create_components(
    def: &GameDefinition,
    id: building::TypeId,
    position: Position,
    rotation: Matrix,
) -> Components {
    let building = def.get_building(id);

    (
        Id::new(rand::random()),
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
    )
}

/// Save type for nodes.
pub mod save {
    use std::collections::BTreeMap;

    use super::*;
    use crate::def;
    use crate::units;

    /// Saves all data related to a node.
    #[derive(Serialize, Deserialize)]
    pub struct Node {
        pub(crate) id: super::Id,
        pub(crate) name: super::Name,
        pub(crate) position: Position,
        pub(crate) shape: Shape,
        pub(crate) cargo: BTreeMap<def::cargo::TypeId, units::Portion<units::CargoSize>>,
        pub(crate) liquid: BTreeMap<def::liquid::TypeId, units::Portion<units::LiquidVolume>>,
        pub(crate) gas: BTreeMap<def::gas::TypeId, units::Portion<units::GasVolume>>,
        pub(crate) hitpoints: units::Portion<units::Hitpoint>,
    }
}
