use std::cell::Cell;
use std::rc::Rc;

use crate::camera::Camera;
use traffloat::shape::{Shape, Texture};
use traffloat::types::{ConfigStore, Position, Vector};

mod canvas;
pub use canvas::*;
mod able;
pub use able::*;
mod image;
pub use image::*;
mod perf;
pub use perf::*;

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
#[read_component(Renderable)]
#[read_component(Position)]
#[read_component(Shape)]
#[allow(clippy::too_many_arguments)]
pub fn render(
    world: &legion::world::SubWorld,
    #[state] canvas_flag: &mut RenderFlag,
    #[state] perf: &mut Rc<Perf>,
    #[state] image_store: &mut ImageStore,
    #[state] render_fps: &mut fps::Counter,
    #[state] simul_fps: &mut fps::Counter,
    #[resource] camera: &Camera,
    #[resource] textures: &ConfigStore<Texture>,
) {
    use legion::IntoQuery;

    let simul_fps = simul_fps.add_frame();

    let canvas = match canvas_flag.cell.replace(None) {
        Some(canvas) => canvas,
        None => return,
    };

    let render_fps = render_fps.add_frame();

    canvas.reset([0., 0., 0., 1.]);
    canvas.rect(
        (0, 0),
        (canvas.dim.width, canvas.dim.height),
        [0., 0., 0., 1.],
    );

    canvas.note(
        format!(
            "FPS: graphics {}, physics {}, cycle time {} \u{03bc}s",
            render_fps,
            simul_fps,
            perf.average_exec_us()
        ),
        (10, 20),
        [1., 1., 1., 1.],
    );
    canvas.note(
        format!(
            "Position: ({:.1}, {:.1})",
            &camera.position[0], &camera.position[1]
        ),
        (10, 40),
        [1., 1., 1., 1.],
    );

    let projection = camera.projection(canvas.dim);

    for (&position, shape, _) in <(&Position, &Shape, &Renderable)>::query().iter(world) {
        // projection matrix transforms real coordinates to canvas

        let unit_to_real = shape.transform(position);
        let image = image_store.fetch(shape.texture, shape.texture.get(textures));
        canvas.draw_image(image, projection * unit_to_real);
    }
}

pub fn setup_ecs(
    setup: traffloat::SetupEcs,
    render_flag: &RenderFlag,
    perf: &Rc<Perf>,
) -> traffloat::SetupEcs {
    let id = {
        let mut t = setup.resources.get_mut::<ConfigStore<Texture>>().expect("");
        t.add(Texture {
            url: String::from("SOF3.png"),
        })
    };
    setup
        .system_local(render_system(
            render_flag.clone(),
            Rc::clone(perf),
            ImageStore::default(),
            fps::Counter::default(),
            fps::Counter::default(),
        ))
        .entity((
            Renderable,
            Position::new(1., 2.),
            Shape {
                matrix: traffloat::types::Matrix::identity(),
                texture: id,
            },
        ))
}
