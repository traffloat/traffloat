//! Renders the user interface.

use derive_new::new;

use super::comm::{Comm, RenderFlag};
use crate::camera::Camera;
use crate::config::RENDER_DEBUG;
use crate::util;

use traffloat::sun::Sun;
use traffloat::time;

pub mod fps;

/// Resource type for simulation FPS counter.
#[derive(Default)]
pub struct SimulFps(
    /// The actual FPS counter
    pub fps::Counter,
);

/// Resource type for rendering FPS counter.
#[derive(Default)]
pub struct RenderFps(
    /// The actual FPS counter
    pub fps::Counter,
);

/// Stores setup data for the debug layer.
#[derive(new)]
pub struct Setup {
    writer: util::DebugWriter,
}

#[codegen::system]
#[thread_local]
pub fn draw(
    #[resource] camera: &Camera,
    #[resource] canvas: &Option<super::Canvas>,
    #[resource] clock: &time::Clock,
    #[resource] comm: &mut Comm,
    #[resource] perf_read: &mut codegen::Perf,
    #[resource] render_fps: &mut RenderFps,
    #[resource] simul_fps: &mut SimulFps,
    #[resource] sun: &Sun,
    #[subscriber] render_flag: impl Iterator<Item = RenderFlag>,
) {
    // Store FPS data
    let simul_fps = simul_fps.0.add_frame();

    if !RENDER_DEBUG {
        return;
    }

    match render_flag.last() {
        Some(RenderFlag) => (),
        None => return,
    };
    let mut canvas = match canvas.as_ref() {
        Some(canvas) => canvas.borrow_mut(),
        None => return,
    };
    let writer = &mut canvas.debug_mut().writer;

    let render_fps = render_fps.0.add_frame();

    // Start actual logging
    writer.reset();

    writer.write(format!(
        "FPS: graphics {}, physics {}, cycle time {:.2} \u{03bc}s",
        render_fps,
        simul_fps,
        comm.perf.average_exec_us(),
    ));
    writer.write(format!("Time: {:?} (Sun: {:.3})", clock.now, sun.yaw()));
    writer.write(format!(
        "Focus: ({:.1}, {:.1}, {:.1}); Zoom: {}; Distance: {}",
        camera.focus().x(),
        camera.focus().y(),
        camera.focus().z(),
        camera.zoom(),
        camera.distance(),
    ));

    {
        let perf_read_map = perf_read.map.get_mut().expect("Poisoned Perf");
        #[allow(clippy::cast_precision_loss)]
        for (sys, stats) in perf_read_map {
            let deque = stats.get_mut().expect("Poisoned Perf");
            let avg = deque.iter().map(|&t| t as f64).sum::<f64>() / (deque.len() as f64);
            let sd = (deque.iter().map(|&t| (t as f64 - avg).powi(2)).sum::<f64>()
                / (deque.len() as f64))
                .sqrt();
            writer.write(format!(
                "Cycle time for system {}: {:.2} \u{03bc}s (\u{00b1} {:.4} \u{03bc}s)",
                sys, avg, sd
            ));
        }

        writer.flush();
    }
}

/// Sets up legion ECS for debug info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup)
}
