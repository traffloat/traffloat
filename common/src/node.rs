use super::types::*;

#[derive(Debug, Clone, Copy, Component, codegen::Gen)]
#[storage(storage::VecStorage)]
pub struct NodeId(pub u32);
