//! Shape and appearance of an object

use crate::types::{Config, ConfigStore, Id, Matrix, Position};
use crate::SetupEcs;

/// Describes the shape and appearance of an object
pub struct Shape {
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
