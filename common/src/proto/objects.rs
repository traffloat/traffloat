//! Information abaout nodes

use crate::shape;
use crate::texture;
use crate::types::*;

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
    /// Bounding box model of the node
    pub shape: shape::Shape,
    /// Rendering model of the node
    pub model: texture::Model,
}
