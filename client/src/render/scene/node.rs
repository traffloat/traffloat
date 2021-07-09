//! Node rendering

use web_sys::{WebGlProgram, WebGlRenderingContext};

use super::{mesh, texture};
use crate::render::util::{create_program, glize_matrix, glize_vector, WebglExt};
use safety::Safety;
use traffloat::space::{Matrix, Vector};

/// Stores the setup data for node rendering.
pub struct Program {
    node_prog: WebGlProgram,
    cube: mesh::PreparedMesh,
}

impl Program {
    /// Initializes node canvas resources.
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let node_prog = create_program(
            gl,
            "node.vert",
            include_str!("node.min.vert"),
            "node.frag",
            include_str!("node.min.frag"),
        );
        let cube = mesh::CUBE.prepare(gl);

        Self { node_prog, cube }
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
        gl.use_program(Some(&self.node_prog));
        gl.set_uniform(&self.node_prog, "u_proj", glize_matrix(proj));
        gl.set_uniform(&self.node_prog, "u_sun", glize_vector(sun));
        gl.set_uniform(
            &self.node_prog,
            "u_brightness",
            brightness.lossy_trunc().clamp(0.5, 1.),
        );
        gl.set_uniform(
            &self.node_prog,
            "u_inv_gain",
            if selected { 0.5 } else { 1. },
        );

        self.cube.positions().apply(gl, &self.node_prog, "a_pos");
        self.cube.normals().apply(gl, &self.node_prog, "a_normal");

        texture.apply(
            self.cube.tex_pos(),
            &self.node_prog,
            "a_tex_pos",
            gl.get_uniform_location(&self.node_prog, "u_tex").as_ref(),
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
