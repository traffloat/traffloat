//! Star rendering

use std::convert::TryInto;

use traffloat::space::{LinearMatrix, Vector};
use web_sys::{WebGlProgram, WebGlRenderingContext};

use crate::render::util::{
    create_program, AttrLocation, BufferUsage, FloatBuffer, UniformLocation,
};

/// Stores the setup data for node rendering.
pub struct Program {
    prog:    WebGlProgram,
    pos_buf: FloatBuffer,
    a_pos:   AttrLocation,
    u_trans: UniformLocation<LinearMatrix>,
}

impl Program {
    /// Initializes sun canvas resources.
    pub fn new(gl: &WebGlRenderingContext, seed: [u8; 32]) -> Self {
        let prog = create_program(gl, glsl!("star"));

        let pos_buf =
            FloatBuffer::create(gl, &generate_vertices(seed), 3, BufferUsage::WriteOnceReadMany);

        let a_pos = AttrLocation::new(gl, &prog, "a_pos");
        let u_trans = UniformLocation::new(gl, &prog, "u_trans");

        Self { prog, pos_buf, a_pos, u_trans }
    }

    /// Draws the sun on the scene.
    pub fn draw(&self, gl: &WebGlRenderingContext, trans: LinearMatrix) {
        gl.use_program(Some(&self.prog));
        self.u_trans.assign(gl, trans);
        self.a_pos.assign(gl, &self.pos_buf);
        gl.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            (NUM_STARS * 3).try_into().expect("Buffer is too large"),
        );
    }
}

/// Number of stars to generate.
const NUM_STARS: usize = 8192;
const STAR_SCALE: f64 = 0.001;

fn generate_vertices(seed: [u8; 32]) -> Vec<f32> {
    use rand::SeedableRng;
    use rand_distr::{Distribution, LogNormal, UnitSphere};
    use rand_xoshiro::Xoshiro256StarStar;

    use crate::render::util::Glize;

    let mut rng = Xoshiro256StarStar::from_seed(seed);

    let mut output = Vec::with_capacity(NUM_STARS * 6 * 3);

    let size_distr = LogNormal::new(0., 0.5).expect("STAR_SCALE is finite");

    for _ in 0..NUM_STARS {
        let vertex = UnitSphere.sample(&mut rng);
        let mut vertex = Vector::from_iterator(vertex);

        let axis = loop {
            let axis = UnitSphere.sample(&mut rng);
            let axis = Vector::from_iterator(axis);

            if axis.dot(&vertex).abs() < 0.95 {
                break axis;
            }
        };

        vertex *= 0.999;

        let dir1 = vertex.cross(&axis).normalize() * size_distr.sample(&mut rng) * STAR_SCALE;
        let dir2 = vertex.cross(&dir1).normalize() * size_distr.sample(&mut rng) * STAR_SCALE;

        let v1 = vertex + dir1;
        let v2 = vertex - dir1;
        let v3 = vertex + dir2;
        let v4 = vertex - dir2;

        output.extend(v1.glize().as_slice());
        output.extend(v2.glize().as_slice());
        output.extend(v3.glize().as_slice());

        output.extend(v4.glize().as_slice());
        output.extend(v2.glize().as_slice());
        output.extend(v1.glize().as_slice());
    }
    output
}
