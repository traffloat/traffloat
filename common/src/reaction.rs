//! Reactions allow conversion between substances.

use specs::WorldExt;

use crate::types::*;
use crate::Setup;

/// The attributes of a reaction type
#[derive(Debug, codegen::Gen, Component)]
#[storage(storage::BTreeStorage)]
pub struct Reaction {
    /// The inputs of this reaction
    pub inputs: Vec<WeightedSubstance>,
    /// The outputs of this reaction
    pub outputs: Vec<WeightedSubstance>,
    /// The amount of heat generated in the reaction
    ///
    /// This value can be negative if the reaction is endothermic.
    pub enthalpy: HeatEnergy,
    /// The amount of electricity required to perform this reaction
    ///
    /// This value can be negative if the reaction generates electricity.
    pub power: ElectricPower,
    /// The amount of time required for each reaction cycle
    pub duration: Time,
}

/// An input/output of a reaction.
#[derive(Debug, codegen::Gen)]
pub enum WeightedSubstance {
    /// A liquid input/output
    Liquid(WeightedLiquid),
    /// A cargo input/output
    Cargo(WeightedCargo),
    /// A gas input/output
    Gas(WeightedGas),
}

/// A liquid input/output
#[derive(Debug, codegen::Gen)]
pub struct WeightedLiquid {
    /// The type of liquid
    pub variant: LiquidId,
    /// The amount of liquid
    pub volume: LiquidVolume,
}

/// A cargo input/output
#[derive(Debug, codegen::Gen)]
pub struct WeightedCargo {
    /// The type of cargo
    pub variant: CargoId,
    /// The amount of cargo
    pub size: CargoSize,
}

/// A gas input/output
#[derive(Debug, codegen::Gen)]
pub struct WeightedGas {
    /// The type of gas
    pub variant: GasId,
    /// The amount of gas
    pub volume: GasVolume,
}

/// A reactor building that can perform reactions given the inputs.
#[derive(Debug, Component, codegen::Gen)]
#[storage(storage::BTreeStorage)]
pub struct Reactor {
    /// The reaction provided by this reactor
    pub reaction: ReactionId,
    /// The multiplier for the time required for the reaction
    pub rate: f32,
    /// The amount of time the reactor has spent on the current reaction cycle
    pub progress: Time,
}

/// Initializes the reaction module
pub fn setup_specs((mut world, dispatcher): Setup) -> Setup {
    world.register::<Reaction>();
    world.register::<Reactor>();
    (world, dispatcher)
}
