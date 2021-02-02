use crate::camera::Camera;
use crate::input;
use traffloat::shape::{Shape, Texture};
use traffloat::types::{ConfigStore, Position};

mod canvas;
pub use canvas::*;
mod able;
pub use able::*;
mod image;
pub use image::*;
mod comm;
pub use comm::*;

mod fps;

#[codegen::system]
#[read_component(Renderable)]
#[read_component(Position)]
#[read_component(Shape)]
#[allow(clippy::too_many_arguments)]
#[thread_local]
pub fn render(
    world: &legion::world::SubWorld,
    #[resource] comm: &mut Comm,
    #[state(Default::default())] image_store: &mut ImageStore,
    #[state(Default::default())] render_fps: &mut fps::Counter,
    #[state(Default::default())] simul_fps: &mut fps::Counter,
    #[resource] camera: &mut Camera,
    #[resource] textures: &ConfigStore<Texture>,
    #[resource] cursor: &input::mouse::CursorPosition,
    #[resource] perf_read: &mut codegen::Perf,
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

    // TODO render sun

    let projection = camera.projection(canvas.dim);

    for (&position, shape, _) in <(&Position, &Shape, &Renderable)>::query().iter(world) {
        // projection matrix transforms real coordinates to canvas

        let unit_to_real = shape.transform(position);
        let image = image_store.fetch(shape.texture, shape.texture.get(textures));
        canvas.draw_image(image, projection * unit_to_real);
    }

    let mut y = 20;
    let mut messages = vec![
        format!(
            "FPS: graphics {}, physics {}, cycle time {:.2} \u{03bc}s",
            render_fps,
            simul_fps,
            comm.perf.average_exec_us()
        ),
        format!(
            "Position: ({:.1}, {:.1})",
            camera.position.x(),
            camera.position.y()
        ),
    ];

    #[allow(clippy::cast_precision_loss)]
    for (sys, stats) in perf_read.map.get_mut().expect("Poisoned Perf") {
        let deque = stats.get_mut().expect("Poisoned Perf");
        let avg = deque.iter().map(|&t| t as f64).sum::<f64>() / (deque.len() as f64);
        let sd = (deque.iter().map(|&t| (t as f64 - avg).powi(2)).sum::<f64>()
            / (deque.len() as f64))
            .sqrt();
        messages.push(format!(
            "Cycle time for system {}: {:.2} \u{03bc}s (\u{00b1} {:.4} \u{03bc}s)",
            sys, avg, sd
        ));
    }

    if let Some(pos) = cursor.pos.as_ref() {
        let entity = cursor.entity.as_ref();
        messages.push(format!(
            "Cursor position: ({:.1}, {:.1}) ({:?})",
            pos.x(),
            pos.y(),
            entity
        ));
    }

    for message in messages {
        canvas.note(message, (10, y), [1., 1., 1., 1.]);
        y += 15;
    }
}

pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(render_setup)
}
