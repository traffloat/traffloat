//! Vehicle-related components.

use gusket::Gusket;
use typed_builder::TypedBuilder;

use crate::{units, SetupEcs};

/// A component applied on a node that drives a rail.
#[derive(TypedBuilder, Gusket)]
pub struct RailPump {
    /// The force provided by the pump.
    #[gusket(immut, copy)]
    force: units::RailForce,
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs { setup }
