//! Defines features of a node.

use codegen::Definition;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use traffloat_types::units::{self, Unit};

use crate::catalyst::Catalyst;

pub mod reaction;
pub use reaction::Reaction;
pub mod security;

/// Features of a building.
#[derive(Debug, Clone, Serialize, Deserialize, Definition)]
#[serde(tag = "type")]
pub enum Feature {
    /// The building is a core and must not be destroyed.
    Core,
    /// The building provides housing capacity, and inhabitants can be assigned to it.
    ProvidesHousing(u32),
    /// The building performs a reaction.
    Reaction(Reaction),
    /// The building provides driving force for vehicles on adjacent rails.
    RailPump(PumpSpec<units::RailForce>),
    /// The building provides pumping force for adjacent liquid pipes.
    LiquidPump(PumpSpec<units::PipeForce>),
    /// The building provides pumping force for gas diffusion in adjacent corridors.
    GasPump(PumpSpec<units::FanForce>),
    /// Inhabitants with low skill may not be permitted to enter the node.
    SecureEntry(security::Policy),
    /// Inhabitants with low skill may not be permitted to exit the node.
    SecureExit(security::Policy),
}

/// Describes a generic pump.
#[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize, Definition)]
#[serde(bound = "")]
#[hf_serde(bound = "")]
pub struct PumpSpec<U: Unit + Definition> {
    /// Catalysts affecting the pump efficiency.
    #[getset(get = "pub")]
    catalysts: SmallVec<[Catalyst; 2]>,
    /// The base force provided by the pump.
    #[getset(get_copy = "pub")]
    force:     U,
}
