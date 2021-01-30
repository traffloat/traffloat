use std::cell::Cell;
use std::rc::Rc;

use crate::camera::Camera;

mod canvas;
pub use canvas::{Canvas, Dimension};

mod fps;

/// The state used to store the canvas.
///
/// When rendering is requested, the cell is filled with a Canvas object.
/// The request is fulfilled by setting it to None.
#[derive(Clone, Default)]
pub struct RenderFlag {
    pub cell: Rc<Cell<Option<Canvas>>>,
}

#[legion::system]
#[allow(clippy::indexing_slicing)]
pub fn render(
    #[state] canvas_flag: &mut RenderFlag,
    #[state] render_fps: &mut fps::Counter,
    #[state] simul_fps: &mut fps::Counter,
    #[resource] camera: &mut Camera,
) {
    let simul_fps = simul_fps.add_frame();

    let canvas = match canvas_flag.cell.replace(None) {
        Some(canvas) => canvas,
        None => return,
    };

    let render_fps = render_fps.add_frame();

    canvas.rect(
        (0, 0),
        (canvas.dim.width, canvas.dim.height),
        [0., 0., 0., 1.],
    );

    canvas.note(
        format!("FPS: graphics {}, physics {}", render_fps, simul_fps),
        (10, 20),
        [1., 1., 1., 1.],
    );
    canvas.note(
        format!("Position: ({}, {})", &camera.position[0], &camera.position[1]),
        (10, 50),
        [1., 1., 1., 1.],
    );
}

pub fn setup_ecs(setup: traffloat::SetupEcs, render_flag: &RenderFlag) -> traffloat::SetupEcs {
    setup.system_local(render_system(
        render_flag.clone(),
        fps::Counter::default(),
        fps::Counter::default(),
    ))
}
