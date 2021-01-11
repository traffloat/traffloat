use super::types::*;

/// The endpoints of an edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Component, codegen::Gen)]
#[storage(storage::VecStorage)]
pub struct EdgeId {
    pub first: NodeId,
    pub second: NodeId,
}
