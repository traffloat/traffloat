use std::time::Duration;

use lazy_static::lazy_static;
use web_sys::Performance;

pub struct Timer {
    perf: Performance,
    zero: f64,
}

impl Timer {
    pub fn new() -> Self {
        let window = web_sys::window().unwrap();
        let perf = window.performance().unwrap();
        let zero = perf.now();
        Self { perf, zero }
    }

    pub fn elapsed(&self) -> Duration {
        Duration::from_secs_f64((self.perf.now() - self.zero) * 0.001)
    }
}
