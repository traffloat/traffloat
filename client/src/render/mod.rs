use std::cell::Cell;
use std::rc::Rc;

/// The dimension of a canvas
pub struct Dimension {
    pub width: u32,
    pub height: u32,
}

impl Dimension {
    /// Aspect ratio of the dimension
    pub fn aspect(self) -> f64 {
        (self.width as f64) / (self.height as f64)
    }
}

/// Information for a canvas
pub struct Canvas {
    pub context: web_sys::CanvasRenderingContext2d,
    pub dim: Dimension,
}

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
    let context = match canvas_flag.cell.replace(None) {
        Some(context) => context,
        None => return,
    };
    // TODO
}

pub fn setup_ecs(setup: traffloat::SetupEcs, render_flag: &RenderFlag) -> traffloat::SetupEcs {
    setup.system_local(render_system(render_flag.clone()))
}
