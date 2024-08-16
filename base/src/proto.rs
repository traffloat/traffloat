//! Common network and save types.

use bevy::math::Vec3;
use bevy::transform::components::Transform as BevyTransform;
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

/// A homogeneous rotation transformation.
#[derive(Serialize, Deserialize)]
pub struct Rotation {
    /// Components of the rotation quarternion.
    pub xyzw: [f32; 4],
}

impl Default for Rotation {
    fn default() -> Self { Self { xyzw: bevy::math::Quat::IDENTITY.to_array() } }
}

/// An axis-aligned scaling transformation
#[derive(Serialize, Deserialize)]
pub struct Scale {
    /// Scale factor on the X-axis.
    pub x: f32,
    /// Scale factor on the Y-axis.
    pub y: f32,
    /// Scale factor on the Z-axis.
    pub z: f32,
}

impl Default for Scale {
    fn default() -> Self { Self { x: 1., y: 1., z: 1. } }
}

/// Serializable form of [bevy `Transform`](BevyTransform).
#[derive(Serialize, Deserialize)]
pub struct Transform {
    /// Position transformation.
    pub position: Position,
    /// Rotation transformation.
    pub rotation: Rotation,
    /// Scaling transformation.
    pub scale:    Scale,
}

impl From<BevyTransform> for Transform {
    fn from(value: BevyTransform) -> Self {
        Self {
            position: value.translation.into(),
            rotation: Rotation { xyzw: value.rotation.to_array() },
            scale:    Scale { x: value.scale.x, y: value.scale.y, z: value.scale.z },
        }
    }
}

impl From<Transform> for BevyTransform {
    fn from(value: Transform) -> Self {
        Self {
            translation: value.position.into(),
            rotation:    bevy::math::Quat::from_array(value.rotation.xyzw),
            scale:       Vec3 { x: value.scale.x, y: value.scale.y, z: value.scale.z },
        }
    }
}
