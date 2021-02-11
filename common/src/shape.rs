//! Shape and appearance of an object

use crate::types::{Config, ConfigStore, Id, Matrix, Point, Position};
use crate::SetupEcs;

/// Describes the shape and appearance of an object
pub struct Shape {
    /// Unit shape variant
    pub unit: Unit,
    /// The transformation matrix from the unit square to this shape centered at the
    /// origin
    pub matrix: Matrix,
    /// The texture for rendering the shape
    pub texture: Id<Texture>,
}

impl Shape {
    /// The transformation matrix from the unit square to this shape centered at pos
    pub fn transform(&self, pos: Position) -> Matrix {
        self.matrix.append_translation(&pos.vector())
    }
}

/// A unit shape variant
pub enum Unit {
    /// A unit cube `[0, 1]^3`
    Cube,
    /// A unit sphere `x^2 + y^2 + z^2 <= 1`
    Sphere,
}

impl Unit {
    /// Checks whether the given point is within this unit shape
    pub fn contains(&self, pos: Point) -> bool {
        match self {
            Self::Cube => {
                (0. ..=1.).contains(&pos.x)
                    && (0. ..=1.).contains(&pos.y)
                    && (0. ..=1.).contains(&pos.z)
            }
            Self::Sphere => pos.x.powi(2) + pos.y.powi(2) + pos.z.powi(2) <= 1.,
        }
    }

    /// Computes the axis-aligned bounding box under the given transformation matrix
    ///
    /// The transformation matrix should transform the unit shape to the real coordinates.
    pub fn bb_under(&self, transform: Matrix) -> (Point, Point) {
        fn fmax(a: f64, b: f64) -> f64 {
            if a > b {
                a
            } else {
                b
            }
        }
        fn fmin(a: f64, b: f64) -> f64 {
            if a < b {
                a
            } else {
                b
            }
        }
        match self {
            Self::Cube => {
                todo!("Test 8 points after transformation")
            }
            Self::Sphere => {
                todo!("Use eigenvectors of matrix")
            }
        }
    }
}

/// The texture of a rendered object
#[derive(Debug)]
pub struct Texture {
    /// A URL compatible with `<img src>`
    pub url: String,
}

impl Config for Texture {}

/// Initializes systems
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.resource(ConfigStore::<Texture>::default())
}
