use std::cell::{self, RefCell};
use std::collections::VecDeque;

use safety::Safety;

const PERF_QUEUE_WINDOW: usize = 1000;

/// Tracks performance.
pub struct Perf {
    queue: VecDeque<u64>,
}

impl Perf {
    /// Updates the execution time.
    pub fn push_exec_us(&mut self, time: u64) {
        if self.queue.len() >= PERF_QUEUE_WINDOW {
            self.pop_front();
        }
        self.queue.push_back(time);
    }

    /// Computes the mean execution time of the current window.
    pub fn mean(&self) -> f64 {
        self.queue.iter().sum::<u64>().small_float() / self.queue.len().small_float()
    }
}
