use std::cell::Cell;
use std::rc::Rc;

mod canvas;
pub use canvas::{Canvas, Dimension};

/// The state used to store the canvas.
///
/// When rendering is requested, the cell is filled with a Canvas object.
/// The request is fulfilled by setting it to None.
#[derive(Clone, Default)]
pub struct RenderFlag {
    pub cell: Rc<Cell<Option<Canvas>>>,
}

#[legion::system]
pub fn render(#[state] canvas_flag: &mut RenderFlag) {
    let canvas = match canvas_flag.cell.replace(None) {
        Some(canvas) => canvas,
        None => return,
    };

    canvas.fill_rect(
        (0, 0),
        (canvas.dim.width, canvas.dim.height),
        [0., 0., 0., 1.],
    );
    canvas
        .context
        .fill_rect(0., 0., canvas.dim.width as f64, canvas.dim.height as f64);
}

pub fn setup_ecs(setup: traffloat::SetupEcs, render_flag: &RenderFlag) -> traffloat::SetupEcs {
    setup.system_local(render_system(render_flag.clone()))
}
