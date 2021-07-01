//! Manages the background canvas.

use lazy_static::lazy_static;
use web_sys::{WebGlProgram, WebGlRenderingContext};

use super::util::{self, BufferUsage, FloatBuffer, IndexBuffer, WebglExt};
use super::{Dimension, RenderFlag};
use crate::camera::Camera;
use safety::Safety;
use traffloat::space::{Matrix, Vector};
use traffloat::sun::Sun;

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

/// Sets up the canvas, loading initial data.
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

    let sun_pos_buf = FloatBuffer::create(&gl, &*SUN_MODEL, 2, BufferUsage::WriteOnceReadMany);
    let sun_pos_index_buf = IndexBuffer::create(&gl, &[0, 1, 2, 0, 2, 3, 0, 3, 1], 3);

    Setup {
        gl,
        star_prog,
        sun_prog,
        sun_pos_buf,
        sun_pos_index_buf,
    }
}

/// Stores the setup data of the background canvas.
pub struct Setup {
    gl: WebGlRenderingContext,
    star_prog: WebGlProgram,
    sun_prog: WebGlProgram,
    sun_pos_buf: FloatBuffer,
    sun_pos_index_buf: IndexBuffer,
}

impl Setup {
    /// Resets the scene for the next rendering frame.
    pub fn reset(&self) {
        self.gl.clear_color(0., 0., 0., 1.);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    }

    /// Draws the sun on the scene.
    pub fn draw_sun(&self, screen_pos: Vector, aspect: f32) {
        self.gl.use_program(Some(&self.sun_prog));
        self.gl.set_uniform(
            &self.sun_prog,
            "u_screen_pos",
            util::glize_vector(screen_pos),
        );
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

#[codegen::system]
#[thread_local]
fn draw(
    #[resource(no_init)] dim: &Dimension,
    #[resource] camera: &Camera,
    #[resource] canvas: &Option<super::Canvas>,
    #[resource] sun: &Sun,
    #[subscriber] render_flag: impl Iterator<Item = RenderFlag>,
) {
    // Render flag gate boilerplate
    match render_flag.last() {
        Some(RenderFlag) => (),
        None => return,
    };
    let canvas = match canvas.as_ref() {
        Some(canvas) => canvas.borrow(),
        None => return,
    };

    let bg = canvas.bg();
    bg.reset();

    let sun_pos = sun.direction();
    let screen_pos = camera.projection().transform_vector(&sun_pos);

    bg.draw_sun(screen_pos, dim.aspect().lossy_trunc());
}

/// Sets up legion ECS for debug info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup)
}
