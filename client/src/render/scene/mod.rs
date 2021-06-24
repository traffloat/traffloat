//! Renders nodes, edges and vehicles.

use std::f64::consts::PI;

use lazy_static::lazy_static;
use legion::world::SubWorld;
use web_sys::{WebGlProgram, WebGlRenderingContext};

use super::util::{self, WebglExt};
use super::{ImageStore, RenderFlag};
use crate::camera::Camera;
use crate::util::lerp;
use safety::Safety;
use traffloat::config;
use traffloat::shape::{Shape, Texture};
use traffloat::space::{Matrix, Position, Vector};
use traffloat::sun::{LightStats, Sun, MONTH_COUNT};

mod able;
pub use able::*;

mod mesh;
pub use mesh::*;

lazy_static! {
    static ref CUBE: Mesh = {
        let mut mesh = Mesh::default();

        let coords = &[
            [-1., -1., -1.],
            [-1., -1., 1.],
            [-1., 1., -1.],
            [-1., 1., 1.],
            [1., -1., -1.],
            [1., -1., 1.],
            [1., 1., -1.],
            [1., 1., 1.],
        ];

        // vertex order:
        // 1 2 --> +u
        // 3 4
        //  |
        //  v
        // +v
        let mut push_face = |v1: usize, v2: usize, v3: usize, v4: usize, normal: [f32; 3]| {
            mesh.positions.extend(&coords[v1]);
            mesh.positions.extend(&coords[v2]);
            mesh.positions.extend(&coords[v3]);

            for _ in 0..3 {
                mesh.normals.extend(&normal);
            }

            mesh.tex_pos.extend(&[0., 0., 1., 0., 0., 1.]);

            mesh.positions.extend(&coords[v2]);
            mesh.positions.extend(&coords[v4]);
            mesh.positions.extend(&coords[v3]);

            for _ in 0..3 {
                mesh.normals.extend(&normal);
            }

            mesh.tex_pos.extend(&[1., 0., 1., 1., 0., 1.]);
        };

        // Reference: https://www.khronos.org/opengl/wiki/File:CubeMapAxes.png
        // Positive X
        push_face(0b101, 0b100, 0b111, 0b110, [1., 0., 0.]);
        // Negative X
        push_face(0b000, 0b101, 0b010, 0b011, [-1., 0., 0.]);
        // Positive Y
        push_face(0b011, 0b111, 0b010, 0b110, [0., 1., 0.]);
        // Negative Y
        push_face(0b000, 0b100, 0b001, 0b101, [0., -1., 0.]);
        // Positive Z
        push_face(0b001, 0b101, 0b011, 0b111, [0., 0., 1.]);
        // Negative Z
        push_face(0b100, 0b000, 0b110, 0b010, [0., 0., -1.]);

        mesh
    };
}

/// Sets up the scene canvas.
pub fn setup(gl: WebGlRenderingContext) -> Setup {
    gl.enable(WebGlRenderingContext::DEPTH_TEST);
    gl.enable(WebGlRenderingContext::CULL_FACE);

    let object_prog = util::create_program(
        &gl,
        "object.vert",
        include_str!("object.vert"),
        "object.frag",
        include_str!("object.frag"),
    );

    let cube = CUBE.prepare(&gl);

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
    cube: PreparedMesh,
}

impl Setup {
    /// Clears the canvas.
    pub fn clear(&self) {
        self.gl.clear_color(0., 0., 0., 0.);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    }

    /// Draws an object on the canvas.
    ///
    /// The projection matrix transforms unit model coordinates to projection coordinates directly.
    pub fn draw_object(&self, proj: Matrix, sun: Vector, brightness: f64, texture: &util::Texture) {
        self.gl.use_program(Some(&self.object_prog));
        self.gl
            .set_uniform(&self.object_prog, "u_proj", util::glize_matrix(proj));
        self.gl
            .set_uniform(&self.object_prog, "u_sun", util::glize_vector(sun));
        self.gl
            .set_uniform(&self.object_prog, "u_brightness", brightness.lossy_trunc());
        texture.apply(
            &self.gl,
            &self
                .gl
                .get_uniform_location(&self.object_prog, "u_tex")
                .expect("Uniform not found"),
        );

        self.cube
            .positions()
            .apply(&self.gl, &self.object_prog, "a_pos");
        self.cube
            .normals()
            .apply(&self.gl, &self.object_prog, "a_normal");
        self.cube
            .tex_pos()
            .apply(&self.gl, &self.object_prog, "a_tex_pos");
        self.cube.draw(&self.gl);
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
    #[state(Default::default())] texture_pool: &mut util::TexturePool,
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

        let base_month = sun.yaw() / PI / 2. * MONTH_COUNT as f64;
        #[allow(clippy::indexing_slicing)]
        let brightness = {
            let prev = light.brightness()[base_month.floor() as usize % MONTH_COUNT];
            let next = light.brightness()[base_month.ceil() as usize % MONTH_COUNT];
            lerp(prev, next, base_month.fract())
        };

        let tex: &Texture = shape.texture().get(textures);
        let texture = texture_pool.load(tex.url(), image_store, &scene.gl);

        scene.draw_object(
            projection * unit_to_real,
            sun.direction(),
            brightness,
            &texture,
        );
    }
}

/// Sets up legion ECS for debug info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup)
}
