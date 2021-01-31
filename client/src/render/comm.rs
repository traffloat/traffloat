use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

use super::Canvas;

/// Thread-local communication between yew and legion renderer
#[derive(Clone, Default)]
pub struct Comm {
    /// Render request tracker
    pub flag: Rc<RenderFlag>,
    /// Performance tracker
    pub perf: Rc<Perf>,
}

/// The state used to store the canvas.
#[derive(Default)]
pub struct RenderFlag {
    /// When rendering is requested, the cell is filled with a Canvas object.
    /// The request is fulfilled by setting it to None.
    pub cell: Cell<Option<Canvas>>,
}

/// Performance tracker
#[derive(Default)]
pub struct Perf {
    exec_us: RefCell<VecDeque<u64>>,
}

impl Perf {
    pub fn push_exec_us(&self, time: u64) {
        let mut exec_us = self.exec_us.borrow_mut();
        while exec_us.len() >= 100 {
            exec_us.pop_front();
        }
        exec_us.push_back(time);
    }

    pub fn average_exec_us(&self) -> f64 {
        let exec_us = self.exec_us.borrow();
        #[allow(clippy::cast_precision_loss)]
        {
            exec_us.iter().map(|&us| us as f64).sum::<f64>() / (exec_us.len() as f64)
        }
    }
}
