//! Renders the user interface.

use derive_new::new;

use super::comm::RenderFlag;
use crate::util;

#[cfg(feature = "render-debug")]
pub mod fps;

/// Stores setup data for the debug layer.
#[derive(new)]
pub struct Canvas {
    writer: util::DebugWriter,
}

#[codegen::system(Visualize)]
#[cfg(feature = "render-debug")]
#[thread_local]
fn draw(
    #[resource] layers: &Option<super::Layers>,
    #[resource] perf_read: &mut codegen::Perf,
    #[resource] entries: &codegen::DebugEntries,
    #[subscriber] render_flag: impl Iterator<Item = RenderFlag>,
) {
    match render_flag.last() {
        Some(RenderFlag) => (),
        None => return,
    };
    let mut layers = match layers.as_ref() {
        Some(layers) => layers.borrow_mut(),
        None => return,
    };
    let writer = &mut layers.debug_mut().writer;

    // Start actual logging
    writer.reset();

    for (category, names) in entries.entries() {
        use std::fmt::Write;

        write!(writer, "[{}]", category).expect("String::write_fmt never fails");
        let mut first = true;
        for (name, entry) in names {
            if !first {
                write!(writer, ",").expect("String::write_fmt never fails");
            }
            first = false;
            write!(writer, " {}: {}", name, entry.value().as_ref())
                .expect("String::write_fmt never fails");
        }
        writeln!(writer).expect("String::write_fmt never fails");
    }

    writer.write("");
    writer.write("CYCLE TIME:");
    {
        let perf_read_map = perf_read.map.get_mut().expect("Poisoned Perf");
        #[allow(clippy::cast_precision_loss)]
        for (sys, stats) in perf_read_map {
            let deque = stats.get_mut().expect("Poisoned Perf");
            let avg = deque.iter().map(|&t| t as f64).sum::<f64>() / (deque.len() as f64);
            let sd = (deque.iter().map(|&t| (t as f64 - avg).powi(2)).sum::<f64>()
                / (deque.len() as f64))
                .sqrt();
            let max = deque.iter().map(|&t| t as f64).fold(0., f64::max);
            writer.write(format!(
                "{}: {:.2} \u{03bc}s (\u{00b1} {:.4} \u{03bc}s, \u{2264} {:.4} \u{03bc}s)",
                sys, avg, sd, max
            ));
        }

        writer.flush();
    }
}

/// Sets up legion ECS for debug info rendering.
#[cfg(feature = "render-debug")]
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup).uses(fps::setup_ecs)
}

/// Dummy setup for non-render-debug builds
#[cfg(not(feature = "render-debug"))]
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup
}
