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
    /// Buffer storing vertex positions.
    ///
    /// Corresponds to `a_pos`.
    #[getset(get = "pub")]
    positions: util::FloatBuffer,
    /// Buffer storing vertex normals.
    ///
    /// Corresponds to `a_normal`.
    #[getset(get = "pub")]
    normals: util::FloatBuffer,
    /// Buffer storing texture positions.
    ///
    /// This is a dynamic buffer.
    /// Corresponds to `a_tex_pos`.
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
#[derive(Default)]
pub struct Mesh {
    /// Triplets of floats indicating the unit model position.
    pub positions: Vec<f32>,
    /// Triplets of floats indicating the unit normal of faces.
    ///
    /// Each vector is repeated 3 times.
    pub normals: Vec<f32>,
}

impl Mesh {
    /// Loads the mesh onto a mesh.
    pub fn prepare(&self, gl: &WebGlRenderingContext) -> PreparedMesh {
        let len = self.positions.len() / 3;
        PreparedMesh::builder()
            .positions(util::FloatBuffer::create(
                gl,
                &self.positions,
                3,
                util::BufferUsage::WriteOnceReadMany,
            ))
            .normals(util::FloatBuffer::create(
                gl,
                &self.normals,
                3,
                util::BufferUsage::WriteOnceReadMany,
            ))
            .tex_pos(util::FloatBuffer::create(
                gl,
                &vec![0.; len * 2],
                2,
                util::BufferUsage::WriteManyReadMany,
            ))
            .len(self.positions.len() / 3)
            .build()
    }
}
