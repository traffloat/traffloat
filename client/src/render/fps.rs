use std::collections::VecDeque;

use crate::util;

/// Counts the FPS of the last second
#[derive(Debug, Default)]
pub struct Counter {
    deque: VecDeque<u64>,
}

impl Counter {
    pub fn add_frame(&mut self) -> usize {
        let now = util::real_time();
        let index = match self.deque.binary_search(&(now - 1000)) {
            Ok(index) => index,
            Err(index) => index,
            // index is somewhere between lower and upper bound
            // we don't care about duplicates, so index is good enough
        };
        self.deque.drain(..index);

        self.deque.push_back(now);
        self.deque.len()
    }
}
