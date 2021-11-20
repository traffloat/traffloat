//! Manages the background canvas.

use traffloat::sun::Sun;
use web_sys::WebGlRenderingContext;
use xias::Xias;

use super::{Dimension, RenderFlag};
use crate::camera::Camera;
use crate::options;

pub mod star;
pub mod sun;

/// Stores the setup data of the background canvas.
pub struct Canvas {
    gl:        WebGlRenderingContext,
    star_prog: star::Program,
    sun_prog:  sun::Program,
}

impl Canvas {
    /// Sets up the canvas, loading initial data.
    pub fn new(gl: WebGlRenderingContext, seed: [u8; 32]) -> Self {
        gl.enable(WebGlRenderingContext::CULL_FACE);
        let star_prog = star::Program::new(&gl, seed);
        let sun_prog = sun::Program::new(&gl);

        Self { gl, star_prog, sun_prog }
    }

    /// Resets the scene for the next rendering frame.
    pub fn reset(&self) {
        self.gl.clear_color(0., 0., 0., 1.);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    }
}

#[codegen::system(Visualize)]
#[thread_local]
fn draw(
    #[resource(no_init)] dim: &Dimension,
    #[resource] camera: &Camera,
    #[resource] layers: &Option<super::Layers>,
    #[resource] sun: &Sun,
    #[resource(no_init)] options: &options::Options,
    #[subscriber] render_flag: impl Iterator<Item = RenderFlag>,
) {
    // Render flag gate boilerplate
    match render_flag.last() {
        Some(RenderFlag) => (),
        None => return,
    };
    let layers = match layers.as_ref() {
        Some(layers) => layers.borrow(),
        None => return,
    };

    let bg = layers.bg();
    bg.reset();

    let sun_pos = sun.direction();
    let screen_pos = camera.projection().transform_vector(&sun_pos);

    bg.sun_prog.draw(&bg.gl, screen_pos, dim.aspect().lossy_float());

    if options.graphics().render_stars() {
        bg.star_prog.draw(&bg.gl, camera.asymptotic_projection());
    }
}

/// Sets up legion ECS for debug info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs { setup.uses(draw_setup) }
