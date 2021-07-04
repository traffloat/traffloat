//! Renders user interface.

use derive_new::new;

use super::{Dimension, RenderFlag};

/// Stores setup data for the ui layer.
#[derive(new)]
pub struct Canvas {
    context: web_sys::CanvasRenderingContext2d,
}

impl Canvas {
    /// Resets the canvas.
    pub fn reset(&self, dim: &Dimension) {
        self.context.clear_rect(0., 0., dim.width.into(), dim.height.into());
    }
}

#[codegen::system]
#[thread_local]
fn draw(
    #[resource(no_init)] dim: &Dimension,
    #[resource] canvas: &Option<super::Layers>,
    #[subscriber] render_flag: impl Iterator<Item = RenderFlag>,
) {
    // Render flag gate boilerplate
    match render_flag.last() {
        Some(RenderFlag) => (),
        None => return,
    };
    let canvas = match canvas.as_ref() {
        Some(canvas) => canvas.borrow_mut(),
        None => return,
    };

    let ui = canvas.ui();
    ui.reset(dim);
}

/// Sets up legion ECS for debug info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup)
}
