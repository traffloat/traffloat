use std::f64::consts::PI;

use super::Rate;
use crate::SetupEcs;

/// Scalar configuration values
#[derive(codegen::Gen)]
pub struct ScalarConfig {
    /// The angle the sun moves per tick
    pub sun_speed: Rate<f64>,
}

impl Default for ScalarConfig {
    fn default() -> Self {
        Self {
            sun_speed: Rate(PI * 2. / 300. / 100.), // 5 minutes = 1 year
        }
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.resource(ScalarConfig::default())
}
