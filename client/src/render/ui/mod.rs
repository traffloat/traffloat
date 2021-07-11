//! Renders user interface.

use derive_new::new;

use super::Dimension;

pub mod node;
mod wrapper;
pub use wrapper::*;

/// Stores setup data for the ui layer.
#[derive(new)]
pub struct Canvas {
    context: web_sys::CanvasRenderingContext2d,
}

impl Canvas {
    /// Resets the canvas.
    pub fn reset(&self, dim: &Dimension) {
        self.context
            .clear_rect(0., 0., dim.width.into(), dim.height.into());
    }
}

/// Sets up legion ECS for UI rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(node::setup_ecs)
}
