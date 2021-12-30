//! Handles user input.

use derive_new::new;
use legion::Entity;

pub mod keyboard;
pub mod mouse;

/// A position on the screen.
#[derive(Debug, Clone, Copy, new, gusket::Gusket)]
pub struct ScreenPosition {
    /// X coordinate of the position.
    ///
    /// `0.` indicates left edge and `1.` indicates right edge.
    #[gusket(immut, copy)]
    x: f64,
    /// Y coordinate of the position.
    ///
    /// `0.` indicates top edge and `1.` indicates bottom edge.
    #[gusket(immut, copy)]
    y: f64,
}

/// Resource storing the entity focused.
#[derive(Debug, Clone, Default, gusket::Gusket)]
pub struct FocusTarget {
    /// The focused entity.
    #[gusket(copy)]
    entity: Option<Entity>,
}

/// Sets up legion ECS for input handling.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(keyboard::setup_ecs).uses(mouse::setup_ecs)
}
