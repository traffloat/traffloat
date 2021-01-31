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

    /// Returns the vector from the origin to the position
    #[allow(clippy::indexing_slicing)]
    pub fn vector(&self) -> Vector {
        Vector::new(self.0[0], self.0[1])
    }

    /// Returns the underlying point
    pub fn value(&self) -> Point {
        self.0
    }
}
