//! Node rendering

use web_sys::{WebGlProgram, WebGlRenderingContext};

use super::{mesh, texture};
use crate::render::util::{create_program, UniformLocation};
use safety::Safety;
use traffloat::space::{Matrix, Vector};

/// Stores the setup data for node rendering.
pub struct Program {
    prog: WebGlProgram,
    cube: mesh::PreparedMesh,
    u_proj: UniformLocation<Matrix>,
    u_sun: UniformLocation<Vector>,
    u_brightness: UniformLocation<f64>,
    u_inv_gain: UniformLocation<f32>,
}

impl Program {
    /// Initializes node canvas resources.
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let prog = create_program(
            gl,
            "node.vert",
            include_str!("node.min.vert"),
            "node.frag",
            include_str!("node.min.frag"),
        );
        let cube = mesh::CUBE.prepare(gl);

        let u_proj = UniformLocation::new(gl, &prog, "u_proj");
        let u_sun = UniformLocation::new(gl, &prog, "u_sun");
        let u_brightness = UniformLocation::new(gl, &prog, "u_brightness");
        let u_inv_gain = UniformLocation::new(gl, &prog, "u_inv_gain");

        Self {
            prog,
            cube,
            u_proj,
            u_sun,
            u_brightness,
            u_inv_gain,
        }
    }

    /// Draws a node on the canvas.
    ///
    /// The projection matrix transforms unit model coordinates to projection coordinates directly.
    pub fn draw(
        &self,
        gl: &WebGlRenderingContext,
        proj: Matrix,
        sun: Vector,
        brightness: f64,
        selected: bool,
        texture: &texture::PreparedTexture,
    ) {
        gl.use_program(Some(&self.prog));
        self.u_proj.assign(gl, proj);
        self.u_sun.assign(gl, sun);
        self.u_brightness.assign(gl, brightness.clamp(0.5, 1.));
        self.u_inv_gain
            .assign(gl, if selected { 0.5f32 } else { 1f32 });

        self.cube.positions().apply(gl, &self.prog, "a_pos");
        self.cube.normals().apply(gl, &self.prog, "a_normal");

        texture.apply(
            self.cube.tex_pos(),
            &self.prog,
            "a_tex_pos",
            gl.get_uniform_location(&self.prog, "u_tex").as_ref(),
            gl,
        );

        gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_MAG_FILTER,
            WebGlRenderingContext::NEAREST.homosign(),
        );
        gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_MIN_FILTER,
            WebGlRenderingContext::NEAREST_MIPMAP_NEAREST.homosign(),
        );
        self.cube.draw(gl);
    }
}
