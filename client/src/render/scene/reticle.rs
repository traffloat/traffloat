//! Reticle rendering

use std::convert::TryInto;

use web_sys::{WebGlProgram, WebGlRenderingContext};

use super::mesh;
use crate::render::util::{
    create_program, AttrLocation, BufferUsage, FloatBuffer, UniformLocation,
};
use traffloat::space::Matrix;

/// Stores the setup data for node rendering.
pub struct Program {
    prog: WebGlProgram,
    arrow_buf: FloatBuffer,
    a_pos: AttrLocation,
    u_trans: UniformLocation<Matrix>,
    u_color: UniformLocation<[f32; 3]>,
}

impl Program {
    /// Initializes reticle canvas resources.
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let prog = create_program(gl, glsl!("reticle"));

        let arrow_buf =
            FloatBuffer::create(gl, &mesh::ARROW[..], 3, BufferUsage::WriteOnceReadMany);

        let a_pos = AttrLocation::new(gl, &prog, "a_pos");
        let u_trans = UniformLocation::new(gl, &prog, "u_trans");
        let u_color = UniformLocation::new(gl, &prog, "u_color");

        Self {
            prog,
            arrow_buf,
            a_pos,
            u_trans,
            u_color,
        }
    }

    /// Draws an arrow on the canvas.
    pub fn draw(&self, gl: &WebGlRenderingContext, proj: Matrix, rgb: [f32; 3]) {
        gl.use_program(Some(&self.prog));
        self.u_trans.assign(gl, proj);
        self.u_color.assign(gl, rgb);

        self.a_pos.assign(gl, &self.arrow_buf);
        gl.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            (mesh::ARROW.len() / 3)
                .try_into()
                .expect("Buffer is too large"),
        );
    }
}
