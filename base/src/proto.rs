//! Common network and save types.

use bevy::math::Vec3;
use serde::{Deserialize, Serialize};

/// A generic 3D position.
#[derive(Serialize, Deserialize)]
pub struct Position {
    /// X-coordinate.
    pub x: f32,
    /// Y-coordinate.
    pub y: f32,
    /// Z-coordinate.
    pub z: f32,
}

impl From<Vec3> for Position {
    fn from(value: Vec3) -> Self { Self { x: value.x, y: value.y, z: value.z } }
}

impl From<Position> for Vec3 {
    fn from(value: Position) -> Self { Self { x: value.x, y: value.y, z: value.z } }
}
