use super::edge::EdgeId;
use super::types::*;

#[derive(Debug, codegen::Gen, Component)]
#[storage(storage::VecStorage)]
pub struct Pipe {
    /// The edge on which this pipe is built
    pub edge: EdgeId,
    /// The liquid type transferred through the pipe
    pub variant: LiquidId,
    /// The volume of liquid currently n the pipe
    pub volume: LiquidVolume,
}

#[derive(Debug, codegen::Gen, Component)]
#[storage(storage::BTreeStorage)]
pub struct Liquid {
    pub texture: [f32; 4],
}

#[derive(Debug, codegen::Gen, Component)]
#[storage(storage::BTreeStorage)]
pub struct Refrigerant {
    pub capacity: HeatCapacity,
}
