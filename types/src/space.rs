//! Spatial units

use std::ops::{Add, AddAssign, Sub, SubAssign};

use codegen::{Definition, ResolveContext};
use serde::{Deserialize, Serialize};

/// Standard vector type
pub type Vector = nalgebra::Vector3<f64>;

/// Standard vector type
pub type Point = nalgebra::Point3<f64>;

/// Standard homogenous matrix type
pub type Matrix = nalgebra::Matrix4<f64>;
/// Standard linear transformation matrix type
pub type LinearMatrix = nalgebra::Matrix3<f64>;

/// A component storing the world position of an object.
///
/// This must not be used to represent canvas coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position(pub Point);

impl Position {
    /// Creates a position
    pub fn new(x: f64, y: f64, z: f64) -> Position { Position(Point::new(x, y, z)) }

    /// The X coordinate of the position
    pub fn x(self) -> f64 { self.0.x }
    /// The Y coordinate of the position
    pub fn y(self) -> f64 { self.0.y }
    /// The Z coordinate of the position
    pub fn z(self) -> f64 { self.0.z }

    /// Returns the vector from the origin to the position
    pub fn vector(&self) -> Vector { Vector::new(self.x(), self.y(), self.z()) }

    /// Returns the underlying point
    pub fn value(&self) -> Point { self.0 }
}

impl Sub<Position> for Position {
    type Output = Vector;

    fn sub(self, other: Self) -> Self::Output {
        Vector::new(self.x() - other.x(), self.y() - other.y(), self.z() - other.z())
    }
}

impl Add<Vector> for Position {
    type Output = Position;

    fn add(self, other: Vector) -> Self::Output {
        Position::new(self.x() + other.x, self.y() + other.y, self.z() + other.z)
    }
}
impl AddAssign<Vector> for Position {
    fn add_assign(&mut self, other: Vector) { *self = *self + other; }
}
impl Sub<Vector> for Position {
    type Output = Position;

    fn sub(self, other: Vector) -> Self::Output {
        Position::new(self.x() - other.x, self.y() - other.y, self.z() - other.z)
    }
}
impl SubAssign<Vector> for Position {
    fn sub_assign(&mut self, other: Vector) { *self = *self - other; }
}

/// Creates a transformation matrix from a cube to a cuboid at `lower..upper`.
pub fn transform_cuboid(lower: Vector, upper: Vector) -> Matrix {
    let origin = (lower + upper) / 2.;
    Matrix::new_nonuniform_scaling(&(upper - origin)).append_translation(&origin)
}

/// Creates a transformation matrix from a unit cylinder cuboid
/// to one with base center `(0, 0, -zn)` and top center `(0, 0, zn)`,
/// with cross-section ellipse radii `x`, `y` on the X, Y axes.
pub fn transform_cylinder(x: f64, y: f64, zn: f64, zp: f64) -> Matrix {
    Matrix::new_nonuniform_scaling(&Vector::new(x, y, zn + zp))
        .append_translation(&Vector::new(0., 0., -zn))
}

/// A transformation matrix used in object schema.
///
/// This just wraps the [`Matrix`] type,
/// but it implements [`codegen::Definition`] manually
/// to allow expressing the transformation
/// as a sequence of primitives in TOML format.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TransformMatrix(pub Matrix);

impl Definition for TransformMatrix {
    type HumanFriendly = Vec<TransformPrimitive>;

    fn convert(primitives: Self::HumanFriendly, _: &mut ResolveContext) -> anyhow::Result<Self> {
        let mut matrix = Matrix::identity();
        for primitive in primitives {
            matrix = primitive.to_matrix() * matrix;
        }
        Ok(Self(matrix))
    }
}

/// A primitive linear transformation operation.
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransformPrimitive {
    /// A translation operation.
    Translate {
        /// The distance to translate along the X axis. Default 0.
        #[serde(default)]
        x: f64,
        /// The distance to translate along the Y axis. Default 0.
        #[serde(default)]
        y: f64,
        /// The distance to translate along the Z axis. Default 0.
        #[serde(default)]
        z: f64,
    },
    /// A scaling operation.
    Scale {
        /// The ratio to scale along the X axis. Default 1.
        #[serde(default = "serde_one")]
        x: f64,
        /// The ratio to scale along the Y axis. Default 1.
        #[serde(default = "serde_one")]
        y: f64,
        /// The ratio to scale along the Z axis. Default 1.
        #[serde(default = "serde_one")]
        z: f64,
    },
}

impl TransformPrimitive {
    /// Represent this primitive as a transformation matrix.
    pub fn to_matrix(&self) -> Matrix {
        match *self {
            Self::Translate { x, y, z } => Matrix::new_translation(&Vector::new(x, y, z)),
            Self::Scale { x, y, z } => Matrix::new_nonuniform_scaling(&Vector::new(x, y, z)),
        }
    }
}

fn serde_one() -> f64 { 1. }
