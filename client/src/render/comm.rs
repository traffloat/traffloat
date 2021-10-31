use std::cell::RefCell;
use std::collections::VecDeque;
use std::ops::Deref;
use std::rc::Rc;

/// Thread-local communication between yew and legion renderer
#[derive(Clone, Default)]
pub struct Comm {
    inner: Rc<CommInner>,
}

impl Deref for Comm {
    type Target = CommInner;

    fn deref(&self) -> &CommInner { &*self.inner }
}

/// The actual fields of Comm
#[derive(Default)]
pub struct CommInner {
    /// Performance tracker
    pub perf: Perf,
}

/// Resource for the cursor icon.
#[derive(getset::Getters, getset::Setters)]
pub struct CursorType {
    /// The CSS cursor type.
    #[getset(get = "pub")]
    #[getset(set = "pub")]
    name: &'static str,
}

impl Default for CursorType {
    fn default() -> Self { Self { name: "initial" } }
}

/// An event fired when rendering is requested.
pub struct RenderFlag;

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
