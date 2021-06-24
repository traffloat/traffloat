use std::convert::TryInto;

use typed_builder::TypedBuilder;
use web_sys::WebGlRenderingContext;

use crate::render::util;
use safety::Safety;

/// A complex object to render, uploaded onto a WebGL context.
#[derive(getset::Getters, TypedBuilder)]
pub struct PreparedMesh {
    /// Number of vertices in the mesh.
    #[getset(get = "pub")]
    len: usize,
    /// The position buffer.
    #[getset(get = "pub")]
    positions: util::FloatBuffer,
    /// The position buffer.
    #[getset(get = "pub")]
    normals: util::FloatBuffer,
    /// The position buffer.
    #[getset(get = "pub")]
    tex_pos: util::FloatBuffer,
}

impl PreparedMesh {
    /// Draws the whole mesh on the canvas.
    pub fn draw(&self, gl: &WebGlRenderingContext) {
        gl.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            self.len.try_into().expect("Buffer is too large"),
        );
    }
}

/// An in-memory complex object.
#[derive(Default, TypedBuilder)]
pub struct Mesh {
    /// Triplets of floats indicating the unit model position.
    pub positions: Vec<f32>,
    /// Triplets of floats indicating the unit normal of faces.
    ///
    /// Each vector is repeated 3 times.
    pub normals: Vec<f32>,
    /// Pairs of floats indicating the texture position.
    pub tex_pos: Vec<f32>,
}

impl Mesh {
    /// Loads the mesh onto a mesh.
    pub fn prepare(&self, gl: &WebGlRenderingContext) -> PreparedMesh {
        PreparedMesh::builder()
            .positions(util::FloatBuffer::create(gl, &self.positions, 3))
            .normals(util::FloatBuffer::create(gl, &self.normals, 3))
            .tex_pos(util::FloatBuffer::create(gl, &self.tex_pos, 2))
            .len(self.positions.len() / 3)
            .build()
    }
}
