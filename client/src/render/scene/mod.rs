//! Renders nodes, edges and vehicles.

use std::f64::consts::PI;

use legion::Entity;
use legion::component;
use legion::world::SubWorld;
use web_sys::{WebGlProgram, WebGlRenderingContext};

use super::util::{self, WebglExt};
use super::RenderFlag;
use crate::camera::Camera;
use crate::util::lerp;
use safety::Safety;
use traffloat::config;
use crate::input::mouse;
use traffloat::graph;
use traffloat::shape::{Shape, Texture};
use traffloat::space::{Matrix, Position, Vector};
use traffloat::sun::{LightStats, Sun, MONTH_COUNT};

pub mod cube;
use cube::CUBE;
pub mod cylinder;
use cylinder::CYLINDER;

mod mesh;
pub use mesh::*;

mod texture;

/// Stores the setup data of the scene canvas.
pub struct Canvas {
    gl: WebGlRenderingContext,
    node_prog: WebGlProgram,
    edge_prog: WebGlProgram,
    cube: PreparedMesh,
    cylinder: PreparedIndexedMesh,
}

impl Canvas {
    /// Sets up the scene canvas.
    pub fn new(gl: WebGlRenderingContext) -> Self {
        gl.enable(WebGlRenderingContext::DEPTH_TEST);
        gl.enable(WebGlRenderingContext::CULL_FACE);
        gl.enable(WebGlRenderingContext::BLEND);
        gl.blend_func_separate(
            WebGlRenderingContext::SRC_ALPHA,
            WebGlRenderingContext::ONE_MINUS_SRC_ALPHA,
            WebGlRenderingContext::SRC_ALPHA,
            WebGlRenderingContext::ONE,
        );

        let node_prog = util::create_program(
            &gl,
            "node.vert",
            include_str!("node.min.vert"),
            "node.frag",
            include_str!("node.min.frag"),
        );

        let edge_prog = util::create_program(
            &gl,
            "edge.vert",
            include_str!("edge.min.vert"),
            "edge.frag",
            include_str!("edge.min.frag"),
        );

        let cube = CUBE.prepare(&gl);
        let cylinder = CYLINDER.prepare(&gl);

        Self {
            gl,
            node_prog,
            edge_prog,
            cube,
            cylinder,
        }
    }

    /// Clears the canvas.
    pub fn clear(&self) {
        self.gl.clear_color(0., 0., 0., 0.);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    }

    /// Draws a node on the canvas.
    ///
    /// The projection matrix transforms unit model coordinates to projection coordinates directly.
    pub fn draw_node(
        &self,
        proj: Matrix,
        sun: Vector,
        brightness: f64,
        selected: bool,
        texture: &texture::PreparedTexture,
    ) {
        self.gl.use_program(Some(&self.node_prog));
        self.gl
            .set_uniform(&self.node_prog, "u_proj", util::glize_matrix(proj));
        self.gl
            .set_uniform(&self.node_prog, "u_sun", util::glize_vector(sun));
        self.gl
            .set_uniform(&self.node_prog, "u_brightness", brightness.lossy_trunc().clamp(0.5, 1.));
        self.gl
            .set_uniform(&self.node_prog, "u_inv_gain", if selected { 0.5 } else { 1. });

        self.cube
            .positions()
            .apply(&self.gl, &self.node_prog, "a_pos");
        self.cube
            .normals()
            .apply(&self.gl, &self.node_prog, "a_normal");

        texture.apply(
            self.cube.tex_pos(),
            &self.node_prog,
            "a_tex_pos",
            self.gl
                .get_uniform_location(&self.node_prog, "u_tex")
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

    /// Draws an edge on the canvas.
    pub fn draw_edge(&self, proj: Matrix, sun: Vector, rgba: [f32; 4]) {
        self.gl.use_program(Some(&self.edge_prog));
        self.gl
            .set_uniform(&self.edge_prog, "u_trans", util::glize_matrix(proj));
        self.gl
            .set_uniform(&self.edge_prog, "u_trans_sun", util::glize_vector(sun));
        self.gl.set_uniform(&self.edge_prog, "u_color", rgba);
        self.gl.set_uniform(&self.edge_prog, "u_ambient", 0.3);
        self.gl.set_uniform(&self.edge_prog, "u_diffuse", 0.2);
        self.gl.set_uniform(&self.edge_prog, "u_specular", 1.0);
        self.gl
            .set_uniform(&self.edge_prog, "u_specular_coef", 10.0);

        self.cylinder
            .positions()
            .apply(&self.gl, &self.edge_prog, "a_pos");
        self.cylinder
            .normals()
            .apply(&self.gl, &self.edge_prog, "a_normal");
        self.cylinder.draw(&self.gl);
    }
}

#[codegen::system]
#[read_component(Position)]
#[read_component(Shape)]
#[read_component(LightStats)]
#[read_component(graph::NodeId)]
#[read_component(graph::EdgeId)]
#[read_component(graph::EdgeSize)]
#[thread_local]
fn draw(
    world: &mut SubWorld,
    #[resource] camera: &Camera,
    #[resource] layers: &Option<super::Layers>,
    #[resource] sun: &Sun,
    #[resource] textures: &config::Store<Texture>,
    #[resource] texture_pool: &mut Option<texture::Pool>,
    #[resource] mouse_target: &mouse::Target,
    #[subscriber] render_flag: impl Iterator<Item = RenderFlag>,
) {
    use legion::{EntityStore, IntoQuery};

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

    let sun_dir = sun.direction();

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    for (entity, &position, shape, light) in <(Entity, &Position, &Shape, &LightStats)>::query()
        .filter(component::<graph::NodeId>())
        .iter(world)
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
        let selected = mouse_target.is_entity(entity);

        let tex: &Texture = shape.texture().get(textures);
        let sprite = texture_pool.sprite(tex, &scene.gl);

        scene.draw_node(projection * unit_to_real, sun_dir, brightness, selected, &sprite);
    }

    for (&edge, size) in <(&graph::EdgeId, &graph::EdgeSize)>::query().iter(world) {
        let from = edge.from_entity().expect("from_entity not initialized");
        let to = edge.to_entity().expect("to_entity not initialized");

        let from: Position = *world
            .entry_ref(from)
            .expect("from_entity does not exist")
            .get_component()
            .expect("from node does not have Position");
        let to: Position = *world
            .entry_ref(to)
            .expect("to_entity does not exist")
            .get_component()
            .expect("to node does not have Position");

        let dir = to - from;
        let rot = match nalgebra::Rotation3::rotation_between(&Vector::new(0., 0., 1.), &dir) {
            Some(rot) => rot.to_homogeneous(),
            None => Matrix::identity().append_nonuniform_scaling(&Vector::new(0., 0., -1.)),
        };

        let unit = rot
            .prepend_nonuniform_scaling(&Vector::new(size.radius(), size.radius(), dir.norm()))
            .append_translation(&from.vector());

        scene.draw_edge(
            projection * unit,
            projection.transform_vector(&sun_dir),
            [0.3, 0.5, 0.8, 0.5],
        );
    }
}

/// Sets up legion ECS for debug info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup)
}
