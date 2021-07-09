//! This module generates geometry data for a cube.

use std::f32::consts::PI;

use lazy_static::lazy_static;

use super::IndexedMesh;

/// Number of vertices on each circle of a cylinder
pub const NUM_VERTICES: u16 = 32;

lazy_static! {
    /// A mesh for a unit Z-cylinder (`{(x, y) : x^2 + y^2 = 1} x [0, 1]`)
    /// using [`NUM_VERTICES`] vertices on each of the two circles.
    pub static ref CYLINDER: IndexedMesh = {
        let mut mesh = IndexedMesh::default();

        let mut z = 0.;
        let unit = PI / f32::from(NUM_VERTICES);
        for num in 0..(NUM_VERTICES * 2) {
            let angle = unit * f32::from(num);
            z = 1. - z;

            let cos = angle.cos();
            let sin = angle.sin();
            mesh.positions_mut().extend(&[cos, sin, z]);
            mesh.normals_mut().extend(&[cos, sin, 0.]);
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
}
