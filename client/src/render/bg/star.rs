//! Star rendering

use std::convert::TryInto;

use web_sys::{WebGlProgram, WebGlRenderingContext};

use crate::render::util::{create_program, BufferUsage, FloatBuffer, UniformLocation};
use traffloat::space::{LinearMatrix, Vector};

/// Stores the setup data for node rendering.
pub struct Program {
    prog: WebGlProgram,
    star_pos_buf: FloatBuffer,
    u_trans: UniformLocation<LinearMatrix>,
}

impl Program {
    /// Initializes sun canvas resources.
    pub fn new(gl: &WebGlRenderingContext, seed: [u8; 32]) -> Self {
        let prog = create_program(
            gl,
            "star.vert",
            include_str!("star.min.vert"),
            "star.frag",
            include_str!("star.min.frag"),
        );

        let star_pos_buf = FloatBuffer::create(
            gl,
            &generate_vertices(seed),
            3,
            BufferUsage::WriteOnceReadMany,
        );

        let u_trans = UniformLocation::new(gl, &prog, "u_trans");

        Self {
            prog,
            star_pos_buf,
            u_trans,
        }
    }

    /// Draws the sun on the scene.
    pub fn draw(&self, gl: &WebGlRenderingContext, trans: LinearMatrix) {
        gl.use_program(Some(&self.prog));
        self.u_trans.assign(gl, trans);
        self.star_pos_buf.apply(gl, &self.prog, "a_pos");
        gl.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            (NUM_STARS * 3).try_into().expect("Buffer is too large"),
        );
    }
}

/// Number of stars to generate.
const NUM_STARS: usize = 65536;
const STAR_SCALE: f64 = 0.001;

fn generate_vertices(seed: [u8; 32]) -> Vec<f32> {
    use crate::render::util::Glize;
    use rand::SeedableRng;
    use rand_distr::{Distribution, LogNormal, UnitSphere};
    use rand_xoshiro::Xoshiro256StarStar;

    let mut rng = Xoshiro256StarStar::from_seed(seed);

    let mut output = Vec::with_capacity(NUM_STARS * 6 * 3);

    let size_distr = LogNormal::new(0., 0.5).expect("STAR_SCALE is finite");

    for _ in 0..NUM_STARS {
        let vertex = UnitSphere.sample(&mut rng);

        let vertex = Vector::from_iterator(vertex) * 0.999999;

        let mut axis = Vector::new(1., 0., 0.);
        if axis.dot(&vertex).abs() > 0.9 {
            // the axis and the vertex are almost parallel
            axis = Vector::new(0., 1., 0.);
        }

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
