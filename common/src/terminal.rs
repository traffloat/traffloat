//! Handles terminal buildings.

use specs::WorldExt;

use crate::types::*;
use crate::Setup;

/// A terminal building.
#[derive(Debug, codegen::Gen, Component)]
#[storage(storage::BTreeStorage)]
pub struct Terminal {
    /// The driving force of the terminal on magnetic rails.
    pub rail_force: f32,
    /// The driving force of the terminal on liquid pipes.
    pub pump_force: f32,
}

pub fn setup_specs((mut world, dispatcher): Setup) -> Setup {
    world.register::<Terminal>();
    (world, dispatcher)
}
