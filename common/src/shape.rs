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
    /// A unit square `[0, 1]^2`
    Square,
    /// A unit circle `x^2 + y^2 <= 1`
    Circle,
}

impl Unit {
    /// Checks whether the given point is within this unit shape
    #[allow(clippy::indexing_slicing)]
    pub fn contains(&self, pos: Point) -> bool {
        let x = pos[0];
        let y = pos[1];
        match self {
            Self::Square => (0. ..=1.).contains(&x) && (0. ..=1.).contains(&y),
            Self::Circle => x * x + y * y <= 1.,
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
