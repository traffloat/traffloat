//! Basic node and edge management

use std::collections::BTreeMap;
use std::num::NonZeroUsize;

use legion::Entity;

use crate::SetupEcs;

/// Identifies a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, codegen::Gen)]
pub struct NodeId {
    inner: u32,
}

/// Identifies an edge
#[derive(Debug, Clone, Copy, PartialEq, Eq, codegen::Gen)]
pub struct EdgeId {
    /// The "source" node
    pub from: NodeId,
    /// The "dest" node
    pub to: NodeId,
}

/// Indicates that a node is added
pub struct NodeAddEvent {
    /// The added node
    pub node: NodeId,
}

/// Indicates that a node is flagged for removal
pub struct NodeRemoveEvent {
    /// The added node
    pub node: NodeId,
}

/// Indicates that nodes have been removed
pub struct PostNodeRemoveEvent {
    /// Number of nodes removed
    pub count: NonZeroUsize,
}

/// Indicates that an edge is added
pub struct EdgeAddEvent {
    /// The added node
    pub edge: EdgeId,
}

/// Indicates that an edge is flagged for removal
pub struct EdgeRemoveEvent {
    /// The added node
    pub edge: EdgeId,
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
    #[resource] post_node_remove_pub: &mut shrev::EventChannel<PostNodeRemoveEvent>,
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
        post_node_remove_pub.single_write(PostNodeRemoveEvent { count });
    }

    // queue deletion requests for the next event loop
    for removal in node_removals {
        graph.node_deletion_queue.push(removal.node);
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.resource(Graph::default()).uses(delete_nodes_setup)
}
