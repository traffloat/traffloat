//! Edge rendering

use web_sys::{WebGlProgram, WebGlRenderingContext};

use super::mesh;
use crate::render::util::{create_program, UniformLocation};
use traffloat::space::{Matrix, Vector};

/// Stores the setup data for edge rendering.
pub struct Program {
    prog: WebGlProgram,
    cylinder: mesh::PreparedIndexedMesh,
    u_trans: UniformLocation<Matrix>,
    u_trans_sun: UniformLocation<Vector>,
    u_color: UniformLocation<[f32; 4]>,
    u_ambient: UniformLocation<f32>,
    u_diffuse: UniformLocation<f32>,
    u_specular: UniformLocation<f32>,
    u_specular_coef: UniformLocation<f32>,
}

impl Program {
    /// Initializes edge canvas resources.
    pub fn new(gl: &WebGlRenderingContext) -> Self {
        let prog = create_program(
            gl,
            "edge.vert",
            include_str!("edge.min.vert"),
            "edge.frag",
            include_str!("edge.min.frag"),
        );
        let cylinder = mesh::CYLINDER.prepare(gl);

        let u_trans = UniformLocation::new(gl, &prog, "u_trans");
        let u_trans_sun = UniformLocation::new(gl, &prog, "u_trans_sun");
        let u_color = UniformLocation::new(gl, &prog, "u_color");
        let u_ambient = UniformLocation::new(gl, &prog, "u_ambient");
        let u_diffuse = UniformLocation::new(gl, &prog, "u_diffuse");
        let u_specular = UniformLocation::new(gl, &prog, "u_specular");
        let u_specular_coef = UniformLocation::new(gl, &prog, "u_specular_coef");

        Self {
            prog,
            cylinder,
            u_trans,
            u_trans_sun,
            u_color,
            u_ambient,
            u_diffuse,
            u_specular,
            u_specular_coef,
        }
    }

    /// Draws an edge on the canvas.
    pub fn draw(&self, gl: &WebGlRenderingContext, proj: Matrix, sun: Vector, rgba: [f32; 4]) {
        gl.use_program(Some(&self.prog));
        self.u_trans.assign(gl, proj);
        self.u_trans_sun.assign(gl, sun);
        self.u_color.assign(gl, rgba);
        self.u_ambient.assign(gl, 0.3);
        self.u_diffuse.assign(gl, 0.2);
        self.u_specular.assign(gl, 1.0);
        self.u_specular_coef.assign(gl, 10.0);

        self.cylinder.positions().apply(gl, &self.prog, "a_pos");
        self.cylinder.normals().apply(gl, &self.prog, "a_normal");
        self.cylinder.draw(gl);
    }
}
