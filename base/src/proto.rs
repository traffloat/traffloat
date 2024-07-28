//! Common protobuf types.

use bevy::math::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, prost::Message)]
pub struct Position {
    #[prost(float, tag = "1")]
    x: f32,
    #[prost(float, tag = "2")]
    y: f32,
    #[prost(float, tag = "3")]
    z: f32,
}

impl From<Vec3> for Position {
    fn from(value: Vec3) -> Self { Self { x: value.x, y: value.y, z: value.z } }
}
