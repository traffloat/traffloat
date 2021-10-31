//! Reticle rendering

use std::convert::TryInto;

use traffloat::space::Matrix;
use typed_builder::TypedBuilder;
use web_sys::{WebGlProgram, WebGlRenderingContext};

use crate::render::util::{
    create_program, AttrLocation, BufferUsage, FloatBuffer, UniformLocation,
};

/// Stores the setup data for node rendering.
pub struct Program {
    prog:      WebGlProgram,
    arrow_buf: FloatBuffer,
    arrow_len: usize,
    a_pos:     AttrLocation,
    u_trans:   UniformLocation<Matrix>,
    u_color:   UniformLocation<[f32; 3]>,
}

impl Program {
    /// Initializes reticle canvas resources.
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let prog = create_program(gl, glsl!("reticle"));

        let arrow = make_arrow(ArrowOptions::builder().build());
        let arrow_buf = FloatBuffer::create(gl, &arrow, 3, BufferUsage::WriteOnceReadMany);

        let a_pos = AttrLocation::new(gl, &prog, "a_pos");
        let u_trans = UniformLocation::new(gl, &prog, "u_trans");
        let u_color = UniformLocation::new(gl, &prog, "u_color");

        Self { prog, arrow_buf, arrow_len: arrow.len(), a_pos, u_trans, u_color }
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
            (self.arrow_len / 3).try_into().expect("Buffer is too large"),
        );
    }
}

#[derive(TypedBuilder)]
struct ArrowOptions {
    #[builder(default = 0.01)]
    prism_scale:  f32,
    #[builder(default = 0.04)]
    tip_scale:    f32,
    #[builder(default = 0.8)]
    prism_height: f32,
    #[builder(default = 0.2)]
    tip_height:   f32,
}

fn make_arrow(options: ArrowOptions) -> Vec<f32> {
    let top_corner: [f32; 2] = [0., 2.];
    let left_corner: [f32; 2] = [-(3f32.sqrt()), -1.];
    let right_corner: [f32; 2] = [3f32.sqrt(), -1.];
    let corners = [top_corner, left_corner, right_corner];

    let mut ret = Vec::new();
    for edge in 0..3 {
        let v1 = corners[edge];
        let v2 = corners[(edge + 1) % 3];

        ret.extend(&[0., v1[0] * options.prism_scale, v1[1] * options.prism_scale]);
        ret.extend(&[
            options.prism_height,
            v1[0] * options.prism_scale,
            v1[1] * options.prism_scale,
        ]);
        ret.extend(&[0., v2[0] * options.prism_scale, v2[1] * options.prism_scale]);

        ret.extend(&[
            options.prism_height,
            v2[0] * options.prism_scale,
            v2[1] * options.prism_scale,
        ]);
        ret.extend(&[
            options.prism_height,
            v1[0] * options.prism_scale,
            v1[1] * options.prism_scale,
        ]);
        ret.extend(&[0., v2[0] * options.prism_scale, v2[1] * options.prism_scale]);

        ret.extend(&[options.prism_height + options.tip_height, 0., 0.]);
        ret.extend(&[options.prism_height, -v1[0] * options.tip_scale, -v1[1] * options.tip_scale]);
        ret.extend(&[options.prism_height, -v2[0] * options.tip_scale, -v2[1] * options.tip_scale]);
    }

    ret
}
