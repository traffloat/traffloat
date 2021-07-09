//! Edge rendering

use web_sys::{WebGlProgram, WebGlRenderingContext};

use super::mesh;
use crate::render::util::{create_program, glize_matrix, glize_vector, WebglExt};
use traffloat::space::{Matrix, Vector};

/// Stores the setup data for edge rendering.
pub struct Program {
    edge_prog: WebGlProgram,
    cylinder: mesh::PreparedIndexedMesh,
}

impl Program {
    /// Initializes edge canvas resources.
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let edge_prog = create_program(
            gl,
            "edge.vert",
            include_str!("edge.min.vert"),
            "edge.frag",
            include_str!("edge.min.frag"),
        );
        let cylinder = mesh::CYLINDER.prepare(gl);

        Self {
            edge_prog,
            cylinder,
        }
    }

    /// Draws an edge on the canvas.
    pub fn draw(&self, gl: &WebGlRenderingContext, proj: Matrix, sun: Vector, rgba: [f32; 4]) {
        gl.use_program(Some(&self.edge_prog));
        gl.set_uniform(&self.edge_prog, "u_trans", glize_matrix(proj));
        gl.set_uniform(&self.edge_prog, "u_trans_sun", glize_vector(sun));
        gl.set_uniform(&self.edge_prog, "u_color", rgba);
        gl.set_uniform(&self.edge_prog, "u_ambient", 0.3);
        gl.set_uniform(&self.edge_prog, "u_diffuse", 0.2);
        gl.set_uniform(&self.edge_prog, "u_specular", 1.0);
        gl.set_uniform(&self.edge_prog, "u_specular_coef", 10.0);

        self.cylinder
            .positions()
            .apply(gl, &self.edge_prog, "a_pos");
        self.cylinder
            .normals()
            .apply(gl, &self.edge_prog, "a_normal");
        self.cylinder.draw(gl);
    }
}
