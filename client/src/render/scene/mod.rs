//! Renders nodes, edges and vehicles.

use web_sys::{WebGlProgram, WebGlRenderingContext};

use super::util::{self, WebglExt};
use traffloat::space::Matrix;

mod able;
pub use able::*;

mod mesh;
pub use mesh::*;

/// Sets up the scene canvas.
pub fn setup(gl: WebGlRenderingContext) -> Setup {
    let object_prog = util::create_program(
        &gl,
        "object.vert",
        include_str!("object.vert"),
        "object.frag",
        include_str!("object.frag"),
    );

    let cube = Mesh::builder()
        .positions(util::FloatBuffer::create(
            &gl,
            &[0., 1., 0.5, 1., 0., 0.5, -1., 0., 0.5],
            3,
        ))
        .faces(util::IndexBuffer::create(&gl, &[0, 1, 2], 3))
        .build();

    Setup {
        gl,
        object_prog,
        cube,
    }
}

/// Stores the setup data of the scene canvas.
pub struct Setup {
    gl: WebGlRenderingContext,
    object_prog: WebGlProgram,
    cube: Mesh,
}

impl Setup {
    /// Clears the canvas.
    pub fn clear(&self) {
        self.gl.clear_color(0., 0., 0., 0.);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    }

    /// Draws an object on the canvas.
    pub fn draw_object(&self, proj: Matrix) {
        self.gl.use_program(Some(&self.object_prog));
        // self.gl.set_uniform(&self.object_prog, "u_proj", util::glize_matrix(proj));
        self.gl.set_uniform(
            &self.object_prog,
            "u_proj",
            util::glize_matrix(Matrix::identity()),
        );
        self.cube
            .positions()
            .apply(&self.gl, &self.object_prog, "a_pos");
        self.cube.faces().draw(&self.gl);
    }
}
