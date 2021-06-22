use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::ops::Deref;
use std::rc::Rc;

use super::Canvas;

/// Thread-local communication between yew and legion renderer
#[derive(Clone, Default)]
pub struct Comm {
    inner: Rc<CommInner>,
}

impl Deref for Comm {
    type Target = CommInner;

    fn deref(&self) -> &CommInner {
        &*self.inner
    }
}

/// The actual fields of Comm
pub struct CommInner {
    /// Render request tracker
    pub flag: RenderFlag,
    /// Performance tracker
    pub perf: Perf,
    /// The cursor CSS property for the canvas
    pub canvas_cursor_type: Cell<&'static str>,
}

impl Default for CommInner {
    fn default() -> Self {
        Self {
            flag: RenderFlag::default(),
            perf: Perf::default(),
            canvas_cursor_type: Cell::new("initial"),
        }
    }
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
    /// Adds a sample of execution time.
    pub fn push_exec_us(&self, time: u64) {
        let mut exec_us = self.exec_us.borrow_mut();
        while exec_us.len() >= 100 {
            exec_us.pop_front();
        }
        exec_us.push_back(time);
    }

    /// Computes the average execution time.
    pub fn average_exec_us(&self) -> f64 {
        let exec_us = self.exec_us.borrow();
        #[allow(clippy::cast_precision_loss)]
        {
            exec_us.iter().map(|&us| us as f64).sum::<f64>() / (exec_us.len() as f64)
        }
    }
}
