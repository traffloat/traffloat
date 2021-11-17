//! Shape and appearance of an object

use derive_new::new;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
pub use traffloat_types::geometry::Unit;
use traffloat_types::space::{Matrix, Position};
use typed_builder::TypedBuilder;

use crate::def::atlas;

/// Describes the shape and appearance of an object.
///
/// An object may be composed of multiple components.
#[derive(Debug, Clone, new, getset::Getters, Serialize, Deserialize)]
pub struct Appearance {
    /// The list of components.
    #[getset(get = "pub")]
    components: SmallVec<[Component; 1]>,
}

/// Describes the shape and appearance of an object
#[derive(Debug, Clone, TypedBuilder, getset::CopyGetters, getset::Getters, Serialize)]
pub struct Component {
    #[getset(get_copy = "pub")]
    /// Unit shape variant
    unit:       Unit,
    /// The transformation matrix from the unit shape to this shape centered at the origin.
    #[getset(get_copy = "pub")]
    matrix:     Matrix,
    /// The inverse transformation matrix from this shape centered at the origin to the unit shape.
    #[getset(get_copy = "pub")]
    #[builder(
        default_code = r#"matrix.try_inverse().expect("Transformation matrix is singular")"#
    )]
    #[serde(skip)]
    inv_matrix: Matrix,
    /// The texture for rendering the shape
    #[getset(get = "pub")]
    texture:    atlas::ModelRef,
}

impl<'de> Deserialize<'de> for Component {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Simple {
            unit:    Unit,
            matrix:  Matrix,
            texture: atlas::ModelRef,
        }

        let Simple { unit, matrix, texture } = Simple::deserialize(d)?;
        Ok(Self {
            unit,
            matrix,
            inv_matrix: matrix
                .try_inverse()
                .ok_or_else(|| serde::de::Error::custom("Transformation matrix is singular"))?,
            texture,
        })
    }
}

impl Component {
    /// The transformation matrix from the unit shape to this shape centered at pos
    pub fn transform(&self, pos: Position) -> Matrix {
        self.matrix().append_translation(&pos.vector())
    }

    /// The transformation matrix from this shape centered at pos to the unit shape
    pub fn inv_transform(&self, pos: Position) -> Matrix {
        self.inv_matrix().prepend_translation(&-pos.vector())
    }
}
