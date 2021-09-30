//! This module generates geometry data for a cube.

use lazy_static::lazy_static;

use super::Mesh;
use crate::render::texture::{CubeSprites, RectSprite};

/// Positive or negative
#[derive(Debug, Clone, Copy)]
pub enum Sign {
    /// Positive direction
    Positive,
    /// Negative direction
    Negative,
}

impl Sign {
    /// Returns the number one with the sign.
    pub fn as_float(self) -> f32 {
        match self {
            Self::Positive => 1.,
            Self::Negative => -1.,
        }
    }

    /// Inverts this sign.
    pub fn invert(self) -> Self {
        match self {
            Self::Positive => Self::Negative,
            Self::Negative => Self::Positive,
        }
    }

    /// Inverts this sign in-place.
    pub fn invert_mut(&mut self) { *self = self.invert(); }
}

/// A directed axis.
#[derive(Debug, Clone, Copy)]
pub struct DirectedAxis {
    /// The axis number.
    ///
    /// X = 0, Y = 1, Z = 2.
    pub axis: usize,
    /// The direction.
    pub sign: Sign,
}

/// Positive X direction.
pub const POSITIVE_X: DirectedAxis = DirectedAxis { axis: 0, sign: Sign::Positive };
/// Negative X direction.
pub const NEGATIVE_X: DirectedAxis = DirectedAxis { axis: 0, sign: Sign::Negative };
/// Positive Y direction.
pub const POSITIVE_Y: DirectedAxis = DirectedAxis { axis: 1, sign: Sign::Positive };
/// Negative Y direction.
pub const NEGATIVE_Y: DirectedAxis = DirectedAxis { axis: 1, sign: Sign::Negative };
/// Positive Z direction.
pub const POSITIVE_Z: DirectedAxis = DirectedAxis { axis: 2, sign: Sign::Positive };
/// Negative Z direction.
pub const NEGATIVE_Z: DirectedAxis = DirectedAxis { axis: 2, sign: Sign::Negative };

/// The orientation of a face of a cube.
#[derive(Debug, Clone, Copy)]
pub struct Face {
    /// The direction of the face normal.
    pub normal: DirectedAxis,
    /// The texture down direction.
    pub down:   DirectedAxis,
    /// The texture right direction.
    pub right:  DirectedAxis,
}

impl Face {
    /// The position of the lower right coordinates.
    pub fn lower_right_coords(self) -> [f32; 3] {
        let mut output: [f32; 3] = [0., 0., 0.];
        #[allow(clippy::indexing_slicing)]
        {
            output[self.normal.axis] = self.normal.sign.as_float();
            output[self.down.axis] = self.down.sign.as_float();
            output[self.right.axis] = self.right.sign.as_float();
        }
        output
    }

    /// Flips the face vertically.
    fn flip_down(mut self) -> Self {
        self.down.sign.invert_mut();
        self
    }

    /// Flips the face horizontally.
    fn flip_right(mut self) -> Self {
        self.right.sign.invert_mut();
        self
    }
    /// The position of the upper right coordinates.
    pub fn upper_right_coords(self) -> [f32; 3] { self.flip_down().lower_right_coords() }
    /// The position of the lower left coordinates.
    pub fn lower_left_coords(self) -> [f32; 3] { self.flip_right().lower_right_coords() }
    /// The position of the upper left coordinates.
    pub fn upper_left_coords(self) -> [f32; 3] {
        self.flip_down().flip_right().lower_right_coords()
    }

    /// The normal vector of this face.
    pub fn normal(self) -> [f32; 3] {
        let mut normal = [0., 0., 0.];
        #[allow(clippy::indexing_slicing)]
        {
            normal[self.normal.axis] = self.normal.sign.as_float();
        }
        normal
    }

    /// Extracts the sprite corresponding to this face
    pub fn cube_sprite(self, sprites: CubeSprites) -> RectSprite {
        match self.normal.axis {
            0 => match self.normal.sign {
                Sign::Positive => sprites.xp(),
                Sign::Negative => sprites.xn(),
            },
            1 => match self.normal.sign {
                Sign::Positive => sprites.yp(),
                Sign::Negative => sprites.yn(),
            },
            2 => match self.normal.sign {
                Sign::Positive => sprites.zp(),
                Sign::Negative => sprites.zn(),
            },
            _ => unreachable!(),
        }
    }
}

/// The 6 faces of a cube, according to OpenGL order.
///
/// ![](https://www.khronos.org/opengl/wiki_opengl/images/CubeMapAxes.png)
pub const FACES: [Face; 6] = [
    // Reference: https://www.khronos.org/opengl/wiki/File:CubeMapAxes.png
    Face { normal: POSITIVE_X, down: POSITIVE_Y, right: NEGATIVE_Z },
    Face { normal: NEGATIVE_X, down: POSITIVE_Y, right: POSITIVE_Z },
    Face { normal: POSITIVE_Y, down: NEGATIVE_Z, right: POSITIVE_X },
    Face { normal: NEGATIVE_Y, down: POSITIVE_Z, right: POSITIVE_X },
    Face { normal: POSITIVE_Z, down: POSITIVE_Y, right: POSITIVE_X },
    Face { normal: NEGATIVE_Z, down: POSITIVE_Y, right: NEGATIVE_X },
];

lazy_static! {
    /// A mesh for the 6 faces of a unit cube (`[-1, 1]^3`).
    pub static ref CUBE: Mesh = {
        let mut mesh = Mesh::default();

        for (i, &face) in FACES.iter().enumerate() {
            mesh.positions_mut().extend(&face.upper_left_coords());
            mesh.tex_pos_mut().push((i, 0., 0.));

            mesh.positions_mut().extend(&face.upper_right_coords());
            mesh.tex_pos_mut().push((i, 1., 0.));

            mesh.positions_mut().extend(&face.lower_left_coords());
            mesh.tex_pos_mut().push((i, 0., 1.));

            for _ in 0..3 {
                mesh.normals_mut().extend(&face.normal());
            }


            mesh.positions_mut().extend(&face.upper_right_coords());
            mesh.tex_pos_mut().push((i, 1., 0.));

            mesh.positions_mut().extend(&face.lower_right_coords());
            mesh.tex_pos_mut().push((i, 1., 1.));

            mesh.positions_mut().extend(&face.lower_left_coords());
            mesh.tex_pos_mut().push((i, 0., 1.));

            for _ in 0..3 {
                mesh.normals_mut().extend(&face.normal());
            }
        }

        mesh
    };
}
