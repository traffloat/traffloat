use std::convert::TryInto;

use typed_builder::TypedBuilder;
use web_sys::WebGlRenderingContext;

use crate::render::util;

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
#[derive(Default, getset::Getters, getset::MutGetters)]
pub struct Mesh {
    /// Triplets of floats indicating the unit model position.
    #[getset(get = "pub")]
    #[getset(get_mut = "pub")]
    positions: Vec<f32>,
    /// Triplets of floats indicating the unit normal of faces.
    #[getset(get = "pub")]
    #[getset(get_mut = "pub")]
    normals: Vec<f32>,
}

impl Mesh {
    /// Loads the mesh onto a WebGL context.
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

/// A complex object to render with many repetitive vertices, uploaded onto a WebGL context.
#[derive(getset::Getters, TypedBuilder)]
pub struct PreparedIndexedMesh {
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
    /// Buffer storing vertex indices.
    #[getset(get = "pub")]
    indices: util::IndexBuffer,
}

impl PreparedIndexedMesh {
    /// Draws the whole mesh on the canvas.
    pub fn draw(&self, gl: &WebGlRenderingContext) {
        self.indices.draw(gl);
    }
}

/// An in-memory complex object with many repetitive vertices.
#[derive(Default, getset::Getters, getset::MutGetters)]
pub struct IndexedMesh {
    mesh: Mesh,
    /// Triplets of integers indicating the vertices of triangles in the mesh.
    #[getset(get = "pub")]
    #[getset(get_mut = "pub")]
    indices: Vec<u16>,
}

impl IndexedMesh {
    /// Triplets of floats indicating the unit model position.
    pub fn positions(&self) -> &[f32] {
        self.mesh.positions()
    }

    /// Triplets of floats indicating the unit model position.
    pub fn positions_mut(&mut self) -> &mut Vec<f32> {
        self.mesh.positions_mut()
    }

    /// Triples of floats indicating the unit normal of faces.
    pub fn normals(&self) -> &[f32] {
        self.mesh.normals()
    }

    /// Triples of floats indicating the unit normal of faces.
    pub fn normals_mut(&mut self) -> &mut Vec<f32> {
        self.mesh.normals_mut()
    }

    /// Loads the mesh onto a WebGL context.
    pub fn prepare(&self, gl: &WebGlRenderingContext) -> PreparedIndexedMesh {
        PreparedIndexedMesh::builder()
            .positions(util::FloatBuffer::create(
                gl,
                self.positions(),
                3,
                util::BufferUsage::WriteOnceReadMany,
            ))
            .normals(util::FloatBuffer::create(
                gl,
                self.normals(),
                3,
                util::BufferUsage::WriteOnceReadMany,
            ))
            .indices(util::IndexBuffer::create(gl, &self.indices))
            .build()
    }
}
