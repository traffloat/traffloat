//! This module generates geometry data for a cube.

use std::f32::consts::PI;

use lazy_static::lazy_static;

use super::IndexedMesh;
use crate::render::scene::texture::{CylinderSprites, RectSprite};
use safety::Safety;

/// Number of vertices on each circle of a cylinder
pub const NUM_VERTICES: u16 = 32;
const NUM_VERTICES_USIZE: usize = NUM_VERTICES as usize;

/// Sprite number for the curved face.
pub const FACE_CURVED: usize = 0;
/// Sprite number for the top face.
pub const FACE_TOP: usize = 1;
/// Sprite number for the bottom face.
pub const FACE_BOTTOM: usize = 2;

lazy_static! {
    /// A mesh for a unit Z-cylinder (`{(x, y) : x^2 + y^2 = 1} x [0, 1]`)
    /// using [`NUM_VERTICES`] vertices on each of the two circles.
    pub static ref CYLINDER: IndexedMesh = {
        let mut mesh = IndexedMesh::default();

        let step = 1. / f32::from(NUM_VERTICES * 2);

        let nums = 0..(NUM_VERTICES * 2);
        let z_iter = [0_f32, 1.].iter().copied().cycle();
        // For even number vertices, z = 1
        for (num, z) in nums.zip(z_iter) {
            let proportion = f32::from(num) * step;
            let angle = PI * 2. * proportion;
            let (sin, cos) = angle.sin_cos();
            mesh.positions_mut().extend(&[cos, sin, z]);
            mesh.normals_mut().extend(&[cos, sin, 0.]);
            mesh.tex_pos_mut().push((FACE_CURVED, proportion, z));
        }

        for num in 0..NUM_VERTICES {
            let this = num * 2;
            let next = if num + 1 == NUM_VERTICES {
                0
            } else {
                num * 2 + 2
            };

            mesh.indices_mut().extend(&[this, this + 1, next]);
            mesh.indices_mut().extend(&[next + 1, next, this + 1]);
        }

        mesh
    };

    /// A mesh for a unit Z-cylinder with two ends filled
    pub static ref FUSED_CYLINDER: IndexedMesh = {
        let mut mesh = CYLINDER.clone();

        let start_index = mesh.positions().len() as u16 / 3;
        mesh.positions_mut().extend(CYLINDER.positions());
        mesh.normals_mut().extend(CYLINDER.normals());
        for pos in CYLINDER.positions().chunks(3) {
            let sprite_no = if pos[2] > 0.5 { FACE_TOP } else { FACE_BOTTOM };
            let x = (pos[0] + 1.) / 2.;
            let y = (pos[1] + 1.) / 2.;
            mesh.tex_pos_mut().push((sprite_no, x, y));
        }

        let top_index = mesh.positions().len() as u16 / 3;
        mesh.positions_mut().extend(&[0., 0., 1.]);
        mesh.normals_mut().extend(&[0., 0., 1.]);
        mesh.tex_pos_mut().push((FACE_TOP, 0.5, 0.5));

        let bottom_index = mesh.positions().len() as u16 / 3;
        mesh.positions_mut().extend(&[0., 0., 0.]);
        mesh.normals_mut().extend(&[0., 0., -1.]);
        mesh.tex_pos_mut().push((FACE_BOTTOM, 0.5, 0.5));

        for num in 0..NUM_VERTICES {
            let this = num * 2 + start_index;
            let next = (num * 2 + 2) % (NUM_VERTICES * 2) + start_index;
            mesh.indices_mut().extend(&[this, next, top_index]);
            mesh.indices_mut().extend(&[this + 1, next + 1, bottom_index]);
        }

        mesh
    };
}
