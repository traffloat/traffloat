//! Counts the FPS of the last second.

use std::collections::VecDeque;

use crate::render::{Comm, RenderFlag};
use crate::util;

/// Counts the FPS of the last second
#[derive(Debug, Default)]
pub struct Counter {
    deque: VecDeque<u64>,
}

impl Counter {
    /// Adds a time frame.
    ///
    /// Returns the number of frames in the last second.
    pub fn add_frame(&mut self) -> usize {
        let now = util::high_res_time();
        let index = match self.deque.binary_search(&now.saturating_sub(1000000)) {
            Ok(index) => index,
            Err(index) => index,
            // index is somewhere between lower and upper bound
            // we don't care about duplicates, so index is good enough
        };
        self.deque.drain(..index);

        self.deque.push_back(now);
        self.deque.len()
    }

    /// The number of frames in the last second.
    pub fn frames(&self) -> usize {
        self.deque.len()
    }
}

/// Resource type for simulation FPS counter.
#[derive(Default)]
pub struct Simul(
    /// The actual FPS counter
    pub Counter,
);

/// Resource type for rendering FPS counter.
#[derive(Default)]
pub struct Render(
    /// The actual FPS counter
    pub Counter,
);

#[codegen::system(PreVisualize)]
#[thread_local]
fn update(
    #[resource] render_fps: &mut Render,
    #[resource] simul_fps: &mut Simul,
    #[subscriber] render_flag: impl Iterator<Item = RenderFlag>,
    #[resource] comm: &mut Comm,

    #[debug("FPS", "Graphics")] graphics_debug: &codegen::DebugEntry,
    #[debug("FPS", "Physics")] physics_debug: &codegen::DebugEntry,
    #[debug("FPS", "Cycle time")] cycle_time_debug: &codegen::DebugEntry,
) {
    // Store FPS data
    let simul_fps = simul_fps.0.add_frame();

    match render_flag.last() {
        Some(RenderFlag) => (),
        None => return,
    };

    let render_fps = render_fps.0.add_frame();

    codegen::update_debug!(graphics_debug, "{:.1}", render_fps);
    codegen::update_debug!(physics_debug, "{:.1}", simul_fps);
    codegen::update_debug!(cycle_time_debug, "{:.2} \u{03bc}s", comm.perf.average_exec_us());
}

/// Sets up legion ECS for debug info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(update_setup)
}
