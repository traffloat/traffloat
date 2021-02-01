use crate::camera::Camera;
use crate::input;
use traffloat::shape::{self, Shape, Texture};
use traffloat::types::{ConfigStore, Position, Vector};

mod canvas;
pub use canvas::*;
mod able;
pub use able::*;
mod image;
pub use image::*;
mod comm;
pub use comm::*;

mod fps;

#[legion::system]
#[allow(clippy::indexing_slicing)]
#[read_component(Renderable)]
#[read_component(Position)]
#[read_component(Shape)]
#[allow(clippy::too_many_arguments)]
pub fn render(
    world: &legion::world::SubWorld,
    #[resource] comm: &mut Comm,
    #[state] image_store: &mut ImageStore,
    #[state] render_fps: &mut fps::Counter,
    #[state] simul_fps: &mut fps::Counter,
    #[resource] camera: &mut Camera,
    #[resource] textures: &ConfigStore<Texture>,
    #[resource] cursor: &input::mouse::CursorPosition,
) {
    use legion::IntoQuery;

    let simul_fps = simul_fps.add_frame();

    let canvas = match comm.flag.cell.replace(None) {
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

    let projection = camera.projection(canvas.dim);

    for (&position, shape, _) in <(&Position, &Shape, &Renderable)>::query().iter(world) {
        // projection matrix transforms real coordinates to canvas

        let unit_to_real = shape.transform(position);
        let image = image_store.fetch(shape.texture, shape.texture.get(textures));
        canvas.draw_image(image, projection * unit_to_real);
    }

    canvas.note(
        format!(
            "FPS: graphics {}, physics {}, cycle time {} \u{03bc}s",
            render_fps,
            simul_fps,
            comm.perf.average_exec_us()
        ),
        (10, 20),
        [1., 1., 1., 1.],
    );
    canvas.note(
        format!(
            "Position: ({:.1}, {:.1})",
            camera.position.x(),
            camera.position.y()
        ),
        (10, 40),
        [1., 1., 1., 1.],
    );
    if let Some(pos) = cursor.pos.as_ref() {
        let entity = cursor.entity.as_ref();
        canvas.note(
            format!(
                "Cursor position: ({:.1}, {:.1}) ({:?})",
                pos.x(),
                pos.y(),
                entity
            ),
            (10, 60),
            [1., 1., 1., 1.],
        );
    }
}

pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    let id = {
        let mut t = setup.resources.get_mut::<ConfigStore<Texture>>().expect("");
        t.add(Texture {
            url: String::from("SOF3.png"),
        })
    };
    setup
        .system_local(render_system(
            ImageStore::default(),
            fps::Counter::default(),
            fps::Counter::default(),
        ))
        .entity((
            Renderable,
            input::mouse::Clickable,
            Position::new(1., 2.),
            Shape {
                unit: shape::Unit::Square,
                matrix: traffloat::types::Matrix::identity()
                    .append_translation(&Vector::new(-0.5, -0.5)),
                texture: id,
            },
        ))
}
