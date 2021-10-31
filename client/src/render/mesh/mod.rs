//! Standard generated models.

use nalgebra::{Vector2, Vector3};
use web_sys::WebGlRenderingContext;

use super::util::{BufferUsage, FloatBuffer, IndexBuffer};

pub mod cube;
pub mod cylinder;

#[derive(Default)]
struct Builder {
    pos:     Vec<f32>,
    normal:  Vec<f32>,
    tex_pos: Vec<f32>,
    len:     usize,
    indices: Vec<u16>,
}

impl Builder {
    /// Push a vertex to the builder.
    fn push(&mut self, pos: Vector3<f32>, normal: Vector3<f32>, tex_pos: Vector2<f32>) -> usize {
        self.pos.extend(pos.as_slice());
        self.normal.extend(normal.as_slice());
        self.tex_pos.extend(tex_pos.as_slice());

        let len = self.len;
        self.len += 1;
        len
    }

    fn push_triangle(&mut self, triangle: [usize; 3]) {
        for index in triangle {
            self.indices.push(index.try_into().expect("Mesh is too large"));
        }
    }

    fn compile_unindexed(&self, gl: &WebGlRenderingContext) -> impl Mesh {
        Unindexed {
            position: FloatBuffer::create(gl, &self.pos, 3, BufferUsage::WriteOnceReadMany),
            normal:   FloatBuffer::create(gl, &self.normal, 3, BufferUsage::WriteOnceReadMany),
            tex_pos:  FloatBuffer::create(gl, &self.tex_pos, 2, BufferUsage::WriteOnceReadMany),
            len:      self.len,
        }
    }

    fn compile_indexed(&self, gl: &WebGlRenderingContext) -> impl Mesh {
        Indexed {
            position: FloatBuffer::create(gl, &self.pos, 3, BufferUsage::WriteOnceReadMany),
            normal:   FloatBuffer::create(gl, &self.normal, 3, BufferUsage::WriteOnceReadMany),
            tex_pos:  FloatBuffer::create(gl, &self.tex_pos, 2, BufferUsage::WriteOnceReadMany),
            indices:  IndexBuffer::create(gl, &self.indices),
        }
    }
}

/// A generic model used in node/edge rendering.
pub trait Mesh {
    /// The buffer for vertex positions.
    fn position(&self) -> &FloatBuffer;

    /// The buffer for vertex normals.
    fn normal(&self) -> &FloatBuffer;

    /// The buffer for texture positions.
    fn tex_pos(&self) -> &FloatBuffer;

    /// Draw the mesh.
    ///
    /// Called after attributes have been bound to the buffers.
    fn draw(&self, gl: &WebGlRenderingContext);
}

struct Unindexed {
    position: FloatBuffer,
    normal:   FloatBuffer,
    tex_pos:  FloatBuffer,
    len:      usize,
}

impl Mesh for Unindexed {
    fn position(&self) -> &FloatBuffer { &self.position }

    fn normal(&self) -> &FloatBuffer { &self.normal }

    fn tex_pos(&self) -> &FloatBuffer { &self.tex_pos }

    fn draw(&self, gl: &WebGlRenderingContext) {
        gl.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            self.len.try_into().expect("Mesh is too large"),
        );
    }
}

struct Indexed {
    position: FloatBuffer,
    normal:   FloatBuffer,
    tex_pos:  FloatBuffer,
    indices:  IndexBuffer,
}

impl Mesh for Indexed {
    fn position(&self) -> &FloatBuffer { &self.position }

    fn normal(&self) -> &FloatBuffer { &self.normal }

    fn tex_pos(&self) -> &FloatBuffer { &self.tex_pos }

    fn draw(&self, gl: &WebGlRenderingContext) { self.indices.draw(gl); }
}
