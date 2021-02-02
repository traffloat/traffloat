use std::f64::consts::PI;

use crate::camera::Camera;
use crate::input;
use crate::util;
use traffloat::shape::{Shape, Texture};
use traffloat::sun::{LightStats, Sun, MONTH_COUNT};
use traffloat::types::{Clock, ConfigStore, Position, Vector};

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
#[read_component(LightStats)]
#[allow(clippy::too_many_arguments)]
#[thread_local]
pub fn render(
    world: &legion::world::SubWorld,
    #[resource] comm: &mut Comm,
    #[state(Default::default())] image_store: &mut ImageStore,
    #[state(Default::default())] render_fps: &mut fps::Counter,
    #[state(Default::default())] simul_fps: &mut fps::Counter,
    #[resource] camera: &Camera,
    #[resource] clock: &Clock,
    #[resource] sun: &Sun,
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

    draw_sun(&canvas, sun);

    // TODO render stars

    let projection = camera.projection(canvas.dim);

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    for (&position, shape, light, _) in
        <(&Position, &Shape, &LightStats, &Renderable)>::query().iter(world)
    {
        // projection matrix transforms real coordinates to canvas

        let unit_to_real = shape.transform(position);
        let image = image_store.fetch(shape.texture, shape.texture.get(textures));
        canvas.draw_image(image, projection * unit_to_real);

        let base_month = sun.yaw / PI / 2. * MONTH_COUNT as f64;
        #[allow(clippy::indexing_slicing)]
        let brightness = {
            let brightness_prev = light.brightness[base_month.floor() as usize % MONTH_COUNT];
            let brightness_next = light.brightness[base_month.ceil() as usize % MONTH_COUNT];
            util::lerp(brightness_prev, brightness_next, base_month.fract())
        };

        canvas.rect(
            (0., 0.),
            (1., 1.),
            [0., 0., 0., 0.75 - util::fmin(brightness / 5., 1.)],
        );
    }

    {
        let mut y = 20;
        let mut messages = vec![
            format!(
                "FPS: graphics {}, physics {}, cycle time {:.2} \u{03bc}s",
                render_fps,
                simul_fps,
                comm.perf.average_exec_us(),
            ),
            format!("Time: {:?} (Sun: {:.3})", clock.now, sun.yaw,),
            format!(
                "Position: ({:.1}, {:.1}); Zoom height: {}",
                camera.position.x(),
                camera.position.y(),
                camera.render_height,
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
                entity,
            ));
        }

        for message in messages {
            canvas.note(message, (10, y), [1., 1., 1., 1.]);
            y += 15;
        }
    }
}

fn draw_sun(canvas: &Canvas, sun: &Sun) {
    let screen_size = Vector::new(canvas.dim.width as f64, canvas.dim.height as f64);
    let screen_center = screen_size / 2.;
    let sun_dir = Vector::new(sun.yaw.cos(), -sun.yaw.sin());
    let sun_dist = ((sun.yaw.cos() * canvas.dim.width as f64 / 2.).powi(2)
        + (sun.yaw.sin() * canvas.dim.height as f64 / 2.).powi(2))
    .sqrt();
    let sun_pos = screen_center + sun_dir * sun_dist * 3.;

    let gradient = canvas
        .context
        .create_radial_gradient(
            sun_pos.x,
            sun_pos.y,
            0.,
            sun_pos.x,
            sun_pos.y,
            sun_dist * 4.,
        )
        .expect("Failed to create gradient");
    gradient
        .add_color_stop(0., "rgb(128, 128, 128)")
        .expect("Failed to call addColorStep on CanvasGradient");
    gradient
        .add_color_stop(1., "rgb(0, 0, 0)")
        .expect("Failed to call addColorStep on CanvasGradient");

    canvas.context.set_fill_style(&gradient);
    canvas
        .context
        .fill_rect(0., 0., canvas.dim.width as f64, canvas.dim.height as f64);
}

pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(render_setup)
}
