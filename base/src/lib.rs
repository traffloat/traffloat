//! Common utility framework.

pub mod proto;
pub mod save;
mod state;
pub use state::EmptyState;
pub mod partition;
pub use partition::{EventReaderSystemSet, EventWriterSystemSet};
pub mod debug;
