//! Renders the user interface.

use std::collections::BTreeMap;
use std::sync::Mutex;

use derive_new::new;

use super::comm::Perf;
use crate::camera::Camera;
use crate::util;
use traffloat::sun::Sun;
use traffloat::time;

pub mod fps;

/// Stores setup data for the debug layer.
#[derive(new)]
pub struct Setup {
    writer: util::DebugWriter,
}

impl Setup {
    /// Resets the setup.
    pub fn reset(&self) {
        self.writer.reset();
    }

    // TODO reduce the number of arguments here.
    /// Writes the debug data.
    pub fn draw(
        &self,
        render_fps: usize,
        simul_fps: usize,
        camera: &Camera,
        perf_read: &mut codegen::Perf,
        clock: &time::Clock,
        sun: &Sun,
        comm_perf: &super::Perf,
    ) {
        self.writer.write(format!(
            "FPS: graphics {}, physics {}, cycle time {:.2} \u{03bc}s",
            render_fps,
            simul_fps,
            comm_perf.average_exec_us(),
        ));
        self.writer
            .write(format!("Time: {:?} (Sun: {:.3})", clock.now, sun.yaw()));
        self.writer.write(format!(
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
                self.writer.write(format!(
                    "Cycle time for system {}: {:.2} \u{03bc}s (\u{00b1} {:.4} \u{03bc}s)",
                    sys, avg, sd
                ));
            }
        }
    }
}
