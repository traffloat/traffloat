//! A configuration is the special rules defined by the game host in a world.
//!
//! For example, each texture is a configuration, and each liquid type is a configuration.
//!
//! Configurations are stored as resources in the Legion.
//! They are referenced using IDs.

use std::f64::consts::PI;

use super::time;
use crate::SetupEcs;

/// Scalar configuration values
pub struct Scalar {
    /// The angle the sun moves per tick
    pub sun_speed: time::Rate<f64>,
}

impl Default for Scalar {
    fn default() -> Self {
        Self {
            sun_speed: time::Rate(PI * 2. / 300. / 10.), // 5 minutes = 1 year
        }
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup
}
