use std::f64::consts::PI;

use crate::camera::Camera;
use crate::input;
use crate::util;
use codegen::hrtime;
use traffloat::config;
use traffloat::shape::{Shape, Texture};
use traffloat::space::{Position, Vector};
use traffloat::sun::{LightStats, Sun, MONTH_COUNT};
use traffloat::time;

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
    #[resource] clock: &time::Clock,
    #[resource] sun: &Sun,
    #[resource] dim: &Dimension,
    #[resource] textures: &config::Store<Texture>,
    #[resource] cursor: &input::mouse::CursorPosition,
    #[resource] perf_read: &mut codegen::Perf,
) {
    use legion::IntoQuery;

    let simul_fps = simul_fps.add_frame();

    let canvas = match comm.flag.cell.replace(None) {
        Some(canvas) => canvas,
        None => return,
    };
    let mut canvas = canvas.borrow_mut();

    let render_fps = render_fps.add_frame();

    {
        let perf_start = hrtime();
        {
            canvas.new_frame(dim);
            canvas.draw_bg(0., 0., 0.);
            // TODO draw sun
        }
        perf_read.push(
            concat!(module_path!(), "::render::bg"),
            hrtime() - perf_start,
        );
    }

    {
        let perf_start = hrtime();
        {
            let projection = camera.projection();

            #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
            for (&position, shape, light, _) in
                <(&Position, &Shape, &LightStats, &Renderable)>::query().iter(world)
            {
                // projection matrix transforms real coordinates to canvas

                let unit_to_real = shape.transform(position);
                let image = image_store.fetch(shape.texture(), shape.texture().get(textures));

                let base_month = sun.yaw() / PI / 2. * MONTH_COUNT as f64;
                #[allow(clippy::indexing_slicing)]
                let brightness = {
                    let brightness_prev =
                        light.brightness()[base_month.floor() as usize % MONTH_COUNT];
                    let brightness_next =
                        light.brightness()[base_month.ceil() as usize % MONTH_COUNT];
                    util::lerp(brightness_prev, brightness_next, base_month.fract())
                };

                // TODO draw image on projection * unit_to_real with lighting = brightness
            }
        }
        perf_read.push(
            concat!(module_path!(), "::render::scene"),
            hrtime() - perf_start,
        );
    }

    {
        let perf_start = hrtime();
        {
            canvas.write_debug(format!(
                "FPS: graphics {}, physics {}, cycle time {:.2} \u{03bc}s",
                render_fps,
                simul_fps,
                comm.perf.average_exec_us(),
            ));
            canvas.write_debug(format!("Time: {:?} (Sun: {:.3})", clock.now, sun.yaw()));
            canvas.write_debug(format!(
                "Focus: ({:.1}, {:.1}, {:.1}); Zoom: {}; Distance: {}",
                camera.focus().x(),
                camera.focus().y(),
                camera.focus().z(),
                camera.zoom(),
                camera.distance(),
            ));

            #[allow(clippy::cast_precision_loss)]
            for (sys, stats) in perf_read.map.get_mut().expect("Poisoned Perf") {
                let deque = stats.get_mut().expect("Poisoned Perf");
                let avg = deque.iter().map(|&t| t as f64).sum::<f64>() / (deque.len() as f64);
                let sd = (deque.iter().map(|&t| (t as f64 - avg).powi(2)).sum::<f64>()
                    / (deque.len() as f64))
                    .sqrt();
                canvas.write_debug(format!(
                    "Cycle time for system {}: {:.2} \u{03bc}s (\u{00b1} {:.4} \u{03bc}s)",
                    sys, avg, sd
                ));
            }

            if let Some(pos) = cursor.pos.as_ref() {
                let entity = cursor.entity.as_ref();
                canvas.write_debug(format!(
                    "Cursor position: ({:.1}, {:.1}) ({:?})",
                    pos.x(),
                    pos.y(),
                    entity,
                ));
            }
        }
        perf_read.push(
            concat!(module_path!(), "::render::ui"),
            hrtime() - perf_start,
        );
    }
}

pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(render_setup)
}
