//! The common types imported everywhere.

pub use legion::Entity;

use crate::SetupEcs;

/// Standard vector type
pub type Vector = nalgebra::Vector3<f32>;

/// Standard homogenous matrix type
pub type Matrix = nalgebra::Matrix4<f32>;

mod ids;
pub use ids::*;
mod time;
pub use time::{Clock, Instant, Rate, Time};

/// Initializes types
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    time::setup_ecs(setup)
}
