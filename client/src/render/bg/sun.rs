//! Sun rendering

use lazy_static::lazy_static;
use traffloat::space::Vector;
use web_sys::{WebGlProgram, WebGlRenderingContext};

use crate::render::util::{
    create_program, AttrLocation, BufferUsage, FloatBuffer, IndexBuffer, UniformLocation,
};

#[rustfmt::skip]
// f32::sqrt() is not const yet
lazy_static! {
    static ref SUN_MODEL: [f32; 8] = [
        0.0, 0.0, // origin
        -(3f32.sqrt()), 1.,
        3f32.sqrt(), 1.,
        0., -2.,
    ];
}

/// Stores the setup data for node rendering.
pub struct Program {
    prog:          WebGlProgram,
    pos_buf:       FloatBuffer,
    index_buf:     IndexBuffer,
    a_pos:         AttrLocation,
    u_screen_pos:  UniformLocation<Vector>,
    u_color:       UniformLocation<[f32; 3]>,
    u_body_radius: UniformLocation<f32>,
    u_aura_radius: UniformLocation<f32>,
    u_aspect:      UniformLocation<f32>,
}

impl Program {
    /// Initializes sun canvas resources.
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let prog = create_program(gl, glsl!("sun"));

        let pos_buf = FloatBuffer::create(gl, &*SUN_MODEL, 2, BufferUsage::WriteOnceReadMany);
        #[rustfmt::skip]
        let index_buf = IndexBuffer::create(gl, &[
            0, 1, 2,
            0, 2, 3,
            0, 3, 1,
        ]);

        let a_pos = AttrLocation::new(gl, &prog, "a_pos");
        let u_screen_pos = UniformLocation::new(gl, &prog, "u_screen_pos");
        let u_color = UniformLocation::new(gl, &prog, "u_color");
        let u_body_radius = UniformLocation::new(gl, &prog, "u_body_radius");
        let u_aura_radius = UniformLocation::new(gl, &prog, "u_aura_radius");
        let u_aspect = UniformLocation::new(gl, &prog, "u_aspect");

        Self {
            prog,
            pos_buf,
            index_buf,
            a_pos,
            u_screen_pos,
            u_color,
            u_body_radius,
            u_aura_radius,
            u_aspect,
        }
    }

    /// Draws the sun on the scene.
    pub fn draw(&self, gl: &WebGlRenderingContext, screen_pos: Vector, aspect: f32) {
        gl.use_program(Some(&self.prog));
        self.u_screen_pos.assign(gl, screen_pos);
        self.u_color.assign(gl, [1., 0.94902, 0.929412]); // source: https://habr.com/en/post/479264/
        self.u_body_radius.assign(gl, 0.15);
        self.u_aura_radius.assign(gl, 0.15);
        self.u_aspect.assign(gl, aspect);
        self.a_pos.assign(gl, &self.pos_buf);
        self.index_buf.draw(gl);
    }
}
