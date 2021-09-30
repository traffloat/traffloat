//! Population-related components.

use typed_builder::TypedBuilder;

use crate::SetupEcs;

/// Copmonent applied on nodes.
#[derive(TypedBuilder, getset::CopyGetters)]
pub struct Housing {
    /// Housing capacity of the building.
    #[getset(get_copy = "pub")]
    capacity: u32,
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs { setup }
