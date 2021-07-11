//! Handles user input.

use derive_new::new;
use legion::Entity;

pub mod keyboard;
pub mod mouse;

/// A position on the screen.
#[derive(Debug, Clone, Copy, new, getset::CopyGetters)]
pub struct ScreenPosition {
    /// X coordinate of the position.
    ///
    /// `0.` indicates left edge and `1.` indicates right edge.
    #[getset(get_copy = "pub")]
    x: f64,
    /// Y coordinate of the position.
    ///
    /// `0.` indicates top edge and `1.` indicates bottom edge.
    #[getset(get_copy = "pub")]
    y: f64,
}

/// Resource storing the entity focused.
#[derive(Debug, Clone, Default, getset::CopyGetters, getset::Setters)]
pub struct FocusTarget {
    /// The focused entity.
    #[getset(get_copy = "pub")]
    #[getset(set = "pub")]
    entity: Option<Entity>,
}

/// Sets up legion ECS for input handling.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(keyboard::setup_ecs).uses(mouse::setup_ecs)
}
