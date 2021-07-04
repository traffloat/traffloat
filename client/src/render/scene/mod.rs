//! Renders nodes, edges and vehicles.

use std::f64::consts::PI;

use legion::world::SubWorld;
use web_sys::{WebGlProgram, WebGlRenderingContext};

use super::util::{self, WebglExt};
use super::RenderFlag;
use crate::camera::Camera;
use crate::util::lerp;
use safety::Safety;
use traffloat::config;
use traffloat::shape::{Shape, Texture};
use traffloat::space::{Matrix, Position, Vector};
use traffloat::sun::{LightStats, Sun, MONTH_COUNT};

pub mod cube;
use cube::CUBE;

mod marker;
pub use marker::*;

mod mesh;
pub use mesh::*;

mod texture;

/// Stores the setup data of the scene canvas.
pub struct Canvas {
    gl: WebGlRenderingContext,
    object_prog: WebGlProgram,
    cube: PreparedMesh,
}

impl Canvas {
    /// Sets up the scene canvas.
    pub fn new(gl: WebGlRenderingContext) -> Self {
        gl.enable(WebGlRenderingContext::DEPTH_TEST);
        gl.enable(WebGlRenderingContext::CULL_FACE);

        let object_prog = util::create_program(
            &gl,
            "node.vert",
            include_str!("node.vert"),
            "node.frag",
            include_str!("node.frag"),
        );

        let cube = CUBE.prepare(&gl);

        Self {
            gl,
            object_prog,
            cube,
        }
    }

    /// Clears the canvas.
    pub fn clear(&self) {
        self.gl.clear_color(0., 0., 0., 0.);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    }

    /// Draws an object on the canvas.
    ///
    /// The projection matrix transforms unit model coordinates to projection coordinates directly.
    pub fn draw_object(
        &self,
        proj: Matrix,
        sun: Vector,
        brightness: f64,
        texture: &texture::PreparedTexture,
    ) {
        self.gl.use_program(Some(&self.object_prog));
        self.gl
            .set_uniform(&self.object_prog, "u_proj", util::glize_matrix(proj));
        self.gl
            .set_uniform(&self.object_prog, "u_sun", util::glize_vector(sun));
        self.gl
            .set_uniform(&self.object_prog, "u_brightness", brightness.lossy_trunc());

        self.cube
            .positions()
            .apply(&self.gl, &self.object_prog, "a_pos");
        self.cube
            .normals()
            .apply(&self.gl, &self.object_prog, "a_normal");

        texture.apply(
            self.cube.tex_pos(),
            &self.object_prog,
            "a_tex_pos",
            self.gl
                .get_uniform_location(&self.object_prog, "u_tex")
                .as_ref(),
            &self.gl,
        );

        self.gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_MAG_FILTER,
            WebGlRenderingContext::NEAREST.homosign(),
        );
        self.gl.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_MIN_FILTER,
            WebGlRenderingContext::NEAREST_MIPMAP_NEAREST.homosign(),
        );
        self.cube.draw(&self.gl);
    }
}

#[codegen::system]
#[read_component(Position)]
#[read_component(Shape)]
#[read_component(LightStats)]
#[read_component(RenderNode)]
#[thread_local]
fn draw(
    world: &mut SubWorld,
    #[resource] camera: &Camera,
    #[resource] layers: &Option<super::Layers>,
    #[resource] sun: &Sun,
    #[resource] textures: &config::Store<Texture>,
    #[resource] texture_pool: &mut Option<texture::Pool>,
    #[subscriber] render_flag: impl Iterator<Item = RenderFlag>,
) {
    use legion::IntoQuery;

    // Render flag gate boilerplate
    match render_flag.last() {
        Some(RenderFlag) => (),
        None => return,
    };
    let layers = match layers.as_ref() {
        Some(layers) => layers.borrow_mut(),
        None => return,
    };

    let scene = layers.scene();
    scene.clear();

    let projection = camera.projection();

    let texture_pool = texture_pool.get_or_insert_with(|| texture::Pool::new(&scene.gl));

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    for (&position, shape, light, _) in
        <(&Position, &Shape, &LightStats, &RenderNode)>::query().iter(world)
    {
        // projection matrix transforms real coordinates to canvas

        let unit_to_real = shape.transform(position);

        let base_month = sun.yaw() / PI / 2. * MONTH_COUNT as f64;
        #[allow(clippy::indexing_slicing)]
        let brightness = {
            let prev = light.brightness()[base_month.floor() as usize % MONTH_COUNT];
            let next = light.brightness()[base_month.ceil() as usize % MONTH_COUNT];
            lerp(prev, next, base_month.fract())
        };

        let tex: &Texture = shape.texture().get(textures);
        let sprite = texture_pool.sprite(tex, &scene.gl);

        scene.draw_object(
            projection * unit_to_real,
            sun.direction(),
            brightness,
            &sprite,
        );
    }
}

/// Sets up legion ECS for debug info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup)
}
