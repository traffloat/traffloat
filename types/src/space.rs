//! Spatial units

use std::ops::{Add, AddAssign, Sub, SubAssign};

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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position(pub Point);

impl Position {
    /// Creates a position
    pub fn new(x: f64, y: f64, z: f64) -> Position {
        Position(Point::new(x, y, z))
    }

    /// The X coordinate of the position
    pub fn x(self) -> f64 {
        self.0.x
    }
    /// The Y coordinate of the position
    pub fn y(self) -> f64 {
        self.0.y
    }
    /// The Z coordinate of the position
    pub fn z(self) -> f64 {
        self.0.z
    }

    /// Returns the vector from the origin to the position
    pub fn vector(&self) -> Vector {
        Vector::new(self.x(), self.y(), self.z())
    }

    /// Returns the underlying point
    pub fn value(&self) -> Point {
        self.0
    }
}

impl Sub<Position> for Position {
    type Output = Vector;

    fn sub(self, other: Self) -> Self::Output {
        Vector::new(
            self.x() - other.x(),
            self.y() - other.y(),
            self.z() - other.z(),
        )
    }
}

impl Add<Vector> for Position {
    type Output = Position;

    fn add(self, other: Vector) -> Self::Output {
        Position::new(self.x() + other.x, self.y() + other.y, self.z() + other.z)
    }
}
impl AddAssign<Vector> for Position {
    fn add_assign(&mut self, other: Vector) {
        *self = *self + other;
    }
}
impl Sub<Vector> for Position {
    type Output = Position;

    fn sub(self, other: Vector) -> Self::Output {
        Position::new(self.x() - other.x, self.y() - other.y, self.z() - other.z)
    }
}
impl SubAssign<Vector> for Position {
    fn sub_assign(&mut self, other: Vector) {
        *self = *self - other;
    }
}

/// Creates a transformation matrix from a cube to a cuboid at `lower..upper`.
pub fn transform_cuboid(lower: Vector, upper: Vector) -> Matrix {
    let origin = (lower + upper) / 2.;
    Matrix::new_nonuniform_scaling(&(upper - origin)).append_translation(&origin)
}
