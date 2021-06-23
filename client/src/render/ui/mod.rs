//! Renders user interface.

use super::{Dimension, RenderFlag};

#[codegen::system]
#[thread_local]
pub fn draw(
    #[resource(no_init)] dim: &Dimension,
    #[resource] canvas: &Option<super::Canvas>,
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
    ui.reset_transform()
        .expect("CanvasRenderingContext2d.resetTransform() threw");
    ui.clear_rect(0., 0., dim.width as f64, dim.height as f64);
    ui.set_stroke_style(&"black".into());
    ui.set_fill_style(&"white".into());
    ui.set_font("12px sans-serif");
}

/// Sets up legion ECS for debug info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup)
}
