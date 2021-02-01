use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Standard vector type
pub type Vector = nalgebra::Vector2<f64>;

/// Standard vector type
pub type Point = nalgebra::Point2<f64>;

/// Standard homogenous matrix type
pub type Matrix = nalgebra::Matrix3<f64>;

/// A component storing the world position of an object.
///
/// This must not be used to represent canvas coordinates.
#[derive(Debug, Clone, Copy)]
pub struct Position(pub Point);

impl Position {
    /// Creates a position
    pub fn new(x: f64, y: f64) -> Position {
        Position(Point::new(x, y))
    }

    /// The X coordinate of the position
    pub fn x(self) -> f64 {
        self.0.x
    }
    /// The Y coordinate of the position
    #[allow(clippy::indexing_slicing)]
    pub fn y(self) -> f64 {
        self.0.y
    }

    /// Returns the vector from the origin to the position
    pub fn vector(&self) -> Vector {
        Vector::new(self.x(), self.y())
    }

    /// Returns the underlying point
    pub fn value(&self) -> Point {
        self.0
    }
}

impl Sub<Position> for Position {
    type Output = Vector;

    fn sub(self, other: Self) -> Self::Output {
        Vector::new(self.x() - other.x(), self.y() - other.y())
    }
}

impl Add<Vector> for Position {
    type Output = Position;

    fn add(self, other: Vector) -> Self::Output {
        Position::new(self.x() + other.x, self.y() + other.y)
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
        Position::new(self.x() - other.x, self.y() - other.y)
    }
}
impl SubAssign<Vector> for Position {
    fn sub_assign(&mut self, other: Vector) {
        *self = *self - other;
    }
}
