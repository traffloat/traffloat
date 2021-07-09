//! Reticle rendering

use std::convert::TryInto;

use web_sys::{WebGlProgram, WebGlRenderingContext};

use super::mesh;
use crate::render::util::{create_program, glize_matrix, BufferUsage, FloatBuffer, WebglExt};
use traffloat::space::Matrix;

/// Stores the setup data for node rendering.
pub struct Program {
    reticle_prog: WebGlProgram,
    arrow: FloatBuffer,
}

impl Program {
    /// Initializes reticle canvas resources.
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let reticle_prog = create_program(
            gl,
            "reticle.vert",
            include_str!("reticle.min.vert"),
            "reticle.frag",
            include_str!("reticle.min.frag"),
        );

        let arrow = FloatBuffer::create(gl, &mesh::ARROW[..], 3, BufferUsage::WriteOnceReadMany);

        Self {
            reticle_prog,
            arrow,
        }
    }

    /// Draws an arrow on the canvas.
    pub fn draw(&self, gl: &WebGlRenderingContext, proj: Matrix, rgb: [f32; 3]) {
        gl.use_program(Some(&self.reticle_prog));
        gl.set_uniform(&self.reticle_prog, "u_trans", glize_matrix(proj));
        gl.set_uniform(&self.reticle_prog, "u_color", rgb);

        self.arrow.apply(gl, &self.reticle_prog, "a_pos");
        gl.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            (mesh::ARROW.len() / 3)
                .try_into()
                .expect("Buffer is too large"),
        );
    }
}
