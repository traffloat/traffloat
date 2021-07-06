//! Handles user input.

use derive_new::new;

pub mod keyboard;
pub mod mouse;

/// Mode of input.
#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    /// The default mode. Navigate around objects and choose them.
    Navigation,
    /// Select a position to place an entity.
    Placement,
}

impl Mode {
    /// Whether this mode needs cursor segment computation.
    pub fn needs_cursor_segment(&self) -> bool {
        match self {
            Self::Navigation => true,
            Self::Placement => true,
        }
    }

    /// Whether this mode needs cursor segment computation.
    pub fn needs_cursor_entity(&self) -> bool {
        match self {
            Self::Navigation => true,
            Self::Placement => false,
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::Navigation
    }
}

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

/// Sets up legion ECS for input handling.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(keyboard::setup_ecs).uses(mouse::setup_ecs)
}
