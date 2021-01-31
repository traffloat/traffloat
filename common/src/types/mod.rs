//! The common types imported everywhere.

pub use legion::Entity;

use crate::SetupEcs;

mod config;
pub use config::*;
mod space;
pub use space::*;
mod time;
pub use time::{Clock, Instant, Rate, Time};

/// Initializes types
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(time::setup_ecs)
}
