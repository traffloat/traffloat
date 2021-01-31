#[derive(Debug, Clone, Copy, PartialEq, Eq, codegen::Gen)]
pub struct NodeId {
    inner: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, codegen::Gen)]
pub struct EdgeId {
    pub from: NodeId,
    pub to: NodeId,
}
