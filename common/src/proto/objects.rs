//! Information abaout nodes

use crate::graph::NodeId;

/// Instructs the client to add new nodes
#[derive(codegen::Gen)]
pub struct AddNodes {
    /// The nodes to add
    pub nodes: Vec<Node>,
}

/// A network-encoded node
/// containing all metadata
#[derive(codegen::Gen)]
pub struct Node {
    /// ID of the node
    pub id: NodeId,
}
