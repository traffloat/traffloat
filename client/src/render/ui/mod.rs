//! Renders user interface.

use derive_new::new;

use super::{CursorType, Dimension, RenderFlag};

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

    /// Sets the cursor icon.
    pub fn set_cursor(&self, name: &str) {
        let canvas = self.context.canvas().expect("UI does not have a canvas");
        canvas
            .style()
            .set_property("cursor", name)
            .expect("Failed to set canvas cursor property");
    }
}

#[codegen::system]
#[thread_local]
fn draw(
    #[resource(no_init)] dim: &Dimension,
    #[resource] canvas: &Option<super::Layers>,
    #[resource] cursor_type: &CursorType,
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
    ui.set_cursor(cursor_type.name());
}

/// Sets up legion ECS for debug info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup)
}
