//! Vehicle-related components.

use typed_builder::TypedBuilder;

use crate::units;
use crate::SetupEcs;

/// A component applied on a node that drives a rail.
#[derive(TypedBuilder, getset::CopyGetters)]
pub struct RailPump {
    /// The force provided by the pump.
    #[getset(get_copy = "pub")]
    force: units::RailForce,
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup
}
