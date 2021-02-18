use web_sys::{WebGlProgram, WebGlRenderingContext};

use super::util::{self, WebglExt};
use safety::Safety;
use traffloat::space::Matrix;

pub fn setup(gl: WebGlRenderingContext) -> Setup {
    let star_prog = util::create_program(
        &gl,
        "star.vert",
        include_str!("star.vert"),
        "star.frag",
        include_str!("star.frag"),
    );
    let sun_prog = util::create_program(
        &gl,
        "sun.vert",
        include_str!("sun.vert"),
        "sun.frag",
        include_str!("sun.frag"),
    );

    let sun_pos_buf = util::FloatBuffer::create(
        &gl,
        &[
            0.0,
            0.0, // origin
            -(3f32.sqrt()),
            1.,
            3f32.sqrt(),
            1.,
            0.,
            -2.,
        ],
        2,
    );
    let sun_pos_index_buf = util::IndexBuffer::create(&gl, &[0, 1, 2, 0, 2, 3, 0, 3, 1], 3);

    Setup {
        gl,
        star_prog,
        sun_prog,
        sun_pos_buf,
        sun_pos_index_buf,
    }
}

pub struct Setup {
    gl: WebGlRenderingContext,
    star_prog: WebGlProgram,
    sun_prog: WebGlProgram,
    sun_pos_buf: util::FloatBuffer,
    sun_pos_index_buf: util::IndexBuffer,
}

impl Setup {
    pub fn draw_bg(&self, rot: Matrix, aspect: f32) {
        self.gl.clear_color(0., 0., 0., 1.);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        self.draw_sun(rot, aspect);
    }

    pub fn draw_sun(&self, rot: Matrix, aspect: f32) {
        self.gl.use_program(Some(&self.sun_prog));
        let rot = util::GlMatrix::from_iterator(rot.iter().map(|&f| f.lossy_trunc()));
        self.gl.set_uniform(&self.sun_prog, "u_sun_mat", rot);
        self.gl.set_uniform(
            &self.sun_prog,
            "u_color",
            util::GlVector::new(1., 0.94902, 0.929412), // source: https://habr.com/en/post/479264/
        );
        self.gl.set_uniform(&self.sun_prog, "u_body_radius", 0.15);
        self.gl.set_uniform(&self.sun_prog, "u_aura_radius", 0.15);
        self.gl.set_uniform(&self.sun_prog, "u_aspect", aspect);
        self.sun_pos_buf.apply(&self.gl, &self.sun_prog, "a_pos");
        self.sun_pos_index_buf.draw(&self.gl);
    }
}
