//! Renders nodes, edges and vehicles.

use std::f64::consts::PI;

use legion::world::SubWorld;
use web_sys::{WebGlProgram, WebGlRenderingContext};

use super::util::{self, WebglExt};
use super::{Dimension, ImageStore, RenderFlag};
use crate::camera::Camera;
use crate::util::lerp;
use traffloat::config;
use traffloat::shape::{Shape, Texture};
use traffloat::space::{Matrix, Position};
use traffloat::sun::{LightStats, Sun, MONTH_COUNT};

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

#[codegen::system]
#[read_component(Position)]
#[read_component(Shape)]
#[read_component(LightStats)]
#[read_component(Renderable)]
#[thread_local]
pub fn draw(
    world: &mut SubWorld,
    #[resource] camera: &Camera,
    #[resource] canvas: &Option<super::Canvas>,
    #[resource] sun: &Sun,
    #[resource] textures: &config::Store<Texture>,
    #[state(Default::default())] image_store: &mut ImageStore,
    #[subscriber] render_flag: impl Iterator<Item = RenderFlag>,
) {
    use legion::IntoQuery;

    // Render flag gate boilerplate
    match render_flag.last() {
        Some(RenderFlag) => (),
        None => return,
    };
    let canvas = match canvas.as_ref() {
        Some(canvas) => canvas.borrow_mut(),
        None => return,
    };

    let scene = canvas.scene();
    scene.clear();

    let projection = camera.projection();

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    for (&position, shape, light, _) in
        <(&Position, &Shape, &LightStats, &Renderable)>::query().iter(world)
    {
        // projection matrix transforms real coordinates to canvas

        let unit_to_real = shape.transform(position);
        let image = image_store.fetch(shape.texture(), shape.texture().get(textures));

        let base_month = sun.yaw() / PI / 2. * MONTH_COUNT as f64;
        #[allow(clippy::indexing_slicing)]
        let brightness = {
            let brightness_prev = light.brightness()[base_month.floor() as usize % MONTH_COUNT];
            let brightness_next = light.brightness()[base_month.ceil() as usize % MONTH_COUNT];
            lerp(brightness_prev, brightness_next, base_month.fract())
        };

        // TODO draw image on projection * unit_to_real with lighting = brightness
        canvas.draw_object(projection * unit_to_real);
    }
}

/// Sets up legion ECS for debug info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup)
}
