//! Shape and appearance of an object

use arcstr::ArcStr;
use derive_new::new;
use typed_builder::TypedBuilder;

use crate::space::{Matrix, Position};
use crate::SetupEcs;
pub use traffloat_types::geometry::Unit;

/// Describes the shape and appearance of an object
#[derive(TypedBuilder, getset::CopyGetters, getset::Getters)]
pub struct Shape {
    #[getset(get_copy = "pub")]
    /// Unit shape variant
    unit: Unit,
    /// The transformation matrix from the unit shape to this shape centered at the origin.
    #[getset(get_copy = "pub")]
    matrix: Matrix,
    /// The inverse transformation matrix from this shape centered at the origin to the unit shape.
    #[getset(get_copy = "pub")]
    #[builder(
        default_code = r#"matrix.try_inverse().expect("Transformation matrix is singular")"#
    )]
    inv_matrix: Matrix,
    /// The texture for rendering the shape
    #[getset(get = "pub")]
    texture: Texture,
}

impl Shape {
    /// The transformation matrix from the unit shape to this shape centered at pos
    pub fn transform(&self, pos: Position) -> Matrix {
        self.matrix().append_translation(&pos.vector())
    }

    /// The transformation matrix from this shape centered at pos to the unit shape
    pub fn inv_transform(&self, pos: Position) -> Matrix {
        self.inv_matrix().prepend_translation(&-pos.vector())
    }
}

/// The texture of a rendered object
#[derive(Debug, new, getset::Getters)]
pub struct Texture {
    /// A URL to an image file
    #[getset(get = "pub")]
    url: ArcStr,
    /// The name of the texture.
    #[getset(get = "pub")]
    name: ArcStr,
}

/// Initializes systems
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup
}
