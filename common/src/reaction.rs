use super::types::*;

#[derive(Debug, codegen::Gen, Component)]
#[storage(storage::BTreeStorage)]
pub struct Reaction {
    pub inputs: Vec<WeightedSubstance>,
    pub outputs: Vec<WeightedSubstance>,
    pub enthalpy: HeatEnergy,
    pub power: ElectricPower,
    pub rate: f32,
}

#[derive(Debug, codegen::Gen)]
pub struct WeightedSubstance {
    pub weight: f32,
    pub substance: Substance,
}

#[derive(Debug, codegen::Gen)]
pub enum Substance {
    Liquid(LiquidId),
    Cargo(CargoId),
    Gas(GasId),
}

#[derive(Debug, Component, codegen::Gen)]
#[storage(storage::BTreeStorage)]
pub struct Reractor {
    pub reaction: ReactionId,
    pub rate: f32,
}
