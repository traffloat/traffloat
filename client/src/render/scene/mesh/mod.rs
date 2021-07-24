//! Lazily generates meshes for complex objects.

use std::convert::TryInto;

use typed_builder::TypedBuilder;
use web_sys::WebGlRenderingContext;

use crate::render::util;

pub mod arrow;
pub use arrow::ARROW;
pub mod cube;
pub use cube::CUBE;
pub mod cylinder;
pub use cylinder::{CYLINDER, FUSED_CYLINDER};

/// A generic mesh prepared for WebGL rendering.
pub trait AbstractPreparedMesh {
    /// Buffer storing vertex positions.
    ///
    /// Corresponds to `a_pos`.
    fn positions(&self) -> &util::FloatBuffer;

    /// Buffer storing vertex normals.
    ///
    /// Corresponds to `a_normal`.
    fn normals(&self) -> &util::FloatBuffer;
    /// Buffer storing texture positions.
    ///
    /// Corresponds to `a_tex_pos`.
    fn tex_pos(&self) -> &util::FloatBuffer;
    /// A vector storing the sprite number of each vertex.
    ///
    /// This is used with the actual sprite definition
    /// to populate `tex_offset`.
    fn tex_sprite_number(&self) -> &[usize];
    /// Buffer transferring sprite-specific texture offsets.
    ///
    /// Corresponds to `a_tex_offset`.
    fn tex_offset(&self) -> &util::FloatBuffer;

    /// Draws the whole mesh on the canvas.
    fn draw(&self, gl: &WebGlRenderingContext);
}

macro_rules! impl_mesh {
    ($ty:ty, |$mesh:pat, $gl:pat| $draw:tt) => {
        impl AbstractPreparedMesh for $ty {
            fn positions(&self) -> &util::FloatBuffer {
                &self.positions
            }
            fn normals(&self) -> &util::FloatBuffer {
                &self.normals
            }
            fn tex_pos(&self) -> &util::FloatBuffer {
                &self.tex_pos
            }
            fn tex_sprite_number(&self) -> &[usize] {
                &self.tex_sprite_number
            }
            fn tex_offset(&self) -> &util::FloatBuffer {
                &self.tex_offset
            }

            fn draw(&self, $gl: &WebGlRenderingContext) {
                let $mesh = self;
                $draw
            }
        }
    };
}

/// A complex object to render, uploaded onto a WebGL context.
#[derive(TypedBuilder)]
pub struct PreparedMesh {
    /// Number of vertices in the mesh.
    len: usize,
    /// Buffer storing vertex positions.
    ///
    /// Corresponds to `a_pos`.
    positions: util::FloatBuffer,
    /// Buffer storing vertex normals.
    ///
    /// Corresponds to `a_normal`.
    normals: util::FloatBuffer,
    /// Buffer storing texture positions.
    ///
    /// Corresponds to `a_tex_pos`.
    tex_pos: util::FloatBuffer,
    /// A vector storing the sprite number of each vertex.
    ///
    /// This is used with the actual sprite definition
    /// to populate `tex_offset`.
    tex_sprite_number: Vec<usize>,
    /// Buffer transferring sprite-specific texture offsets.
    ///
    /// Corresponds to `a_tex_offset`.
    tex_offset: util::FloatBuffer,
}

impl_mesh!(PreparedMesh, |mesh, gl| {
    gl.draw_arrays(
        WebGlRenderingContext::TRIANGLES,
        0,
        mesh.len.try_into().expect("Buffer is too large"),
    );
});

/// An in-memory complex object.
#[derive(Default, Debug, Clone, getset::Getters, getset::MutGetters)]
pub struct Mesh {
    /// Triplets of floats indicating the unit model position.
    #[getset(get = "pub")]
    #[getset(get_mut = "pub")]
    positions: Vec<f32>,
    /// Triplets of floats indicating the unit normal from the vertex.
    #[getset(get = "pub")]
    #[getset(get_mut = "pub")]
    normals: Vec<f32>,
    /// Tuples indicating the unit sprite position of the vertice and the internal sprite number.
    ///
    /// For cubes, the sprite number is in the ordre of [`cube::FACES`].
    ///
    /// For cylinders, the sprite number is 0 for curved face and 1/2 for top/bottom faces.
    #[getset(get = "pub")]
    #[getset(get_mut = "pub")]
    tex_pos: Vec<(usize, f32, f32)>,
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
                &flatten_tex_pos(self.tex_pos()),
                2,
                util::BufferUsage::WriteOnceReadMany,
            ))
            .tex_sprite_number(flatten_sprite_number(self.tex_pos()))
            .tex_offset(util::FloatBuffer::create(
                gl,
                &vec![0.; 4 * len],
                4,
                util::BufferUsage::WriteManyReadMany,
            ))
            .len(len)
            .build()
    }
}

/// A complex object to render with many repetitive vertices, uploaded onto a WebGL context.
#[derive(TypedBuilder)]
pub struct PreparedIndexedMesh {
    /// Buffer storing vertex positions.
    ///
    /// Corresponds to `a_pos`.
    positions: util::FloatBuffer,
    /// Buffer storing vertex normals.
    ///
    /// Corresponds to `a_normal`.
    normals: util::FloatBuffer,
    /// Buffer storing texture positions.
    ///
    /// Corresponds to `a_tex_pos`.
    tex_pos: util::FloatBuffer,
    /// A vector storing the sprite number of each vertex.
    ///
    /// This is used with the actual sprite definition
    /// to populate `tex_offset`.
    tex_sprite_number: Vec<usize>,
    /// Buffer transferring sprite-specific texture offsets.
    ///
    /// Corresponds to `a_tex_offset`.
    tex_offset: util::FloatBuffer,
    /// Buffer storing vertex indices.
    indices: util::IndexBuffer,
}

impl_mesh!(PreparedIndexedMesh, |mesh, gl| {
    mesh.indices.draw(gl);
});

/// An in-memory complex object with many repetitive vertices.
#[derive(Default, Debug, Clone, getset::Getters, getset::MutGetters)]
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

    /// Tuples indicating the unit sprite position of the vertice and the internal sprite number.
    ///
    /// See [`Mesh::tex_pos`] for details.
    pub fn tex_pos(&self) -> &[(usize, f32, f32)] {
        self.mesh.tex_pos()
    }

    /// Tuples indicating the unit sprite position of the vertice and the internal sprite number.
    ///
    /// See [`Mesh::tex_pos_mut`] for details.
    pub fn tex_pos_mut(&mut self) -> &mut Vec<(usize, f32, f32)> {
        self.mesh.tex_pos_mut()
    }

    /// Loads the mesh onto a WebGL context.
    pub fn prepare(&self, gl: &WebGlRenderingContext) -> PreparedIndexedMesh {
        let len = self.positions().len() / 3;
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
            .tex_pos(util::FloatBuffer::create(
                gl,
                &flatten_tex_pos(self.tex_pos()),
                2,
                util::BufferUsage::WriteOnceReadMany,
            ))
            .tex_sprite_number(flatten_sprite_number(self.tex_pos()))
            .tex_offset(util::FloatBuffer::create(
                gl,
                &vec![0.; 4 * len],
                4,
                util::BufferUsage::WriteManyReadMany,
            ))
            .indices(util::IndexBuffer::create(gl, &self.indices))
            .build()
    }
}

fn flatten_sprite_number(slice: &[(usize, f32, f32)]) -> Vec<usize> {
    slice.iter().map(|&(number, _, _)| number).collect()
}

fn flatten_tex_pos(slice: &[(usize, f32, f32)]) -> Vec<f32> {
    let mut vec = Vec::with_capacity(slice.len() * 2);
    for &(_, x, y) in slice {
        vec.push(x);
        vec.push(y);
    }
    vec
}
