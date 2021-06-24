//! Handles user input.

pub mod keyboard;

/// Sets up legion ECS for input handling.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(keyboard::setup_ecs)
}
