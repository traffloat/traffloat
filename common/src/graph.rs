//! Basic node and edge management

use std::collections::BTreeMap;
use std::num::NonZeroUsize;

use derive_new::new;
use legion::Entity;

use crate::SetupEcs;

/// Component storing an identifier for a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, new)]
pub struct NodeId {
    inner: u32,
}

/// Component storing the name of the node
#[derive(Debug, new, getset::Getters, getset::Setters)]
pub struct NodeName {
    /// Name of the node
    #[getset(get = "pub")]
    #[getset(set = "pub")]
    name: String,
}

/// Component storing the endpoints of an edge
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, new, getset::CopyGetters, getset::Setters,
)]
pub struct EdgeId {
    /// The "source" node
    #[getset(get_copy = "pub")]
    from: NodeId,
    /// The "source" entity
    #[getset(get_copy = "pub")]
    #[getset(set = "pub")]
    #[new(default)]
    from_entity: Option<Entity>,
    /// The "dest" node
    #[getset(get_copy = "pub")]
    to: NodeId,
    /// The "dest" entity
    #[getset(get_copy = "pub")]
    #[getset(set = "pub")]
    #[new(default)]
    to_entity: Option<Entity>,
}

/// Defines the size of an edge
#[derive(Debug, Clone, Copy, new, getset::CopyGetters)]
pub struct EdgeSize {
    /// The radius of the corridor
    #[getset(get_copy = "pub")]
    radius: f64,
}

/// Indicates that a node is added
#[derive(Debug, new, getset::CopyGetters)]
pub struct NodeAddEvent {
    /// The added node
    #[getset(get_copy = "pub")]
    node: NodeId,
}

/// Indicates that a node is flagged for removal
#[derive(Debug, new, getset::CopyGetters)]
pub struct NodeRemoveEvent {
    /// The removed node
    #[getset(get_copy = "pub")]
    node: NodeId,
}

/// Indicates that nodes have been removed
#[derive(Debug, new, getset::CopyGetters)]
pub struct PostNodeRemoveEvent {
    /// Number of nodes removed
    #[getset(get_copy = "pub")]
    count: NonZeroUsize,
}

/// Indicates that an edge is added
#[derive(Debug, new, getset::Getters)]
pub struct EdgeAddEvent {
    /// The added edge
    #[getset(get = "pub")]
    edge: EdgeId,
}

/// Indicates that an edge is flagged for removal
#[derive(Debug, new, getset::Getters)]
pub struct EdgeRemoveEvent {
    /// The removed edge
    #[getset(get = "pub")]
    edge: EdgeId,
}

/// Tracks the nodes and edges in the world
#[derive(Default)]
pub struct Graph {
    node_index: BTreeMap<NodeId, Entity>,
    node_deletion_queue: Vec<NodeId>,
}

impl Graph {
    /// Retrieves the entity ID for the given node
    pub fn get_node(&self, id: NodeId) -> Option<Entity> {
        self.node_index.get(&id).copied()
    }
}

#[codegen::system]
fn delete_nodes(
    cmd_buf: &mut legion::systems::CommandBuffer,
    #[resource] graph: &mut Graph,
    #[subscriber] node_removals: impl Iterator<Item = NodeRemoveEvent>,
    #[publisher] post_node_remove_pub: impl FnMut(PostNodeRemoveEvent),
) {
    for &node in &graph.node_deletion_queue {
        let entity = graph
            .node_index
            .remove(&node)
            .expect("Removing nonexistent node entity");
        cmd_buf.remove(entity);
    }
    let count = graph.node_deletion_queue.len();
    graph.node_deletion_queue.clear();
    if let Some(count) = NonZeroUsize::new(count) {
        post_node_remove_pub(PostNodeRemoveEvent { count });
    }

    // queue deletion requests for the next event loop
    for removal in node_removals {
        graph.node_deletion_queue.push(removal.node);
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(delete_nodes_setup)
}
