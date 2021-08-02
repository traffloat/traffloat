//! Asteroid-related components.

use crate::SetupEcs;

/// Marker component for nodes that are cores.
pub struct Core;

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup
}
