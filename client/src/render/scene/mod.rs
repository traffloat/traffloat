//! Renders nodes, edges and vehicles.

use std::f64::consts::PI;

use legion::world::SubWorld;
use legion::{component, Entity};
use web_sys::WebGlRenderingContext;

use super::RenderFlag;
use crate::camera::Camera;
use crate::input::mouse;
use crate::util::lerp;
use traffloat::config;
use traffloat::graph;
use traffloat::shape::{Shape, Texture};
use traffloat::space::{Matrix, Position, Vector};
use traffloat::sun::{LightStats, Sun, MONTH_COUNT};

pub mod mesh;

pub mod edge;
pub mod node;
pub mod reticle;

mod texture;

/// Stores the setup data of the scene canvas.
pub struct Canvas {
    gl: WebGlRenderingContext,
    node_prog: node::Program,
    edge_prog: edge::Program,
    reticle_prog: reticle::Program,
}

impl Canvas {
    /// Sets up the scene canvas.
    pub fn new(gl: WebGlRenderingContext) -> Self {
        gl.enable(WebGlRenderingContext::DEPTH_TEST);
        gl.blend_func_separate(
            WebGlRenderingContext::SRC_ALPHA,
            WebGlRenderingContext::ONE_MINUS_SRC_ALPHA,
            WebGlRenderingContext::SRC_ALPHA,
            WebGlRenderingContext::ONE,
        );

        let node_prog = node::Program::new(&gl);
        let edge_prog = edge::Program::new(&gl);
        let reticle_prog = reticle::Program::new(&gl);

        Self {
            gl,
            node_prog,
            edge_prog,
            reticle_prog,
        }
    }

    /// Clears the canvas.
    pub fn clear(&self) {
        self.gl.clear_color(0., 0., 0., 0.);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
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

    scene.gl.enable(WebGlRenderingContext::CULL_FACE);
    scene.gl.enable(WebGlRenderingContext::BLEND);

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

        scene.node_prog.draw(
            &scene.gl,
            projection * unit_to_real,
            sun_dir,
            brightness,
            selected,
            &sprite,
        );
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

        scene.edge_prog.draw(
            &scene.gl,
            projection * unit,
            projection.transform_vector(&sun_dir),
            [0.3, 0.5, 0.8, 0.5],
        );
    }

    /// Shift columns frontward (1 -> 2) or backward (2 -> 1)
    fn shift_axes(mut mat: Matrix, front: bool) -> Matrix {
        #[allow(clippy::branches_sharing_code)] // it is confusing to merge them
        if front {
            mat.swap_columns(0, 1);
            mat.swap_columns(1, 2);
        } else {
            mat.swap_columns(2, 0);
            mat.swap_columns(1, 2);
        }

        mat
    }
    let rot_y = shift_axes(Matrix::identity(), true);
    let rot_z = shift_axes(Matrix::identity(), false);

    let arrow_projection = projection.prepend_translation(&camera.focus().vector());

    scene.gl.disable(WebGlRenderingContext::CULL_FACE);
    scene.gl.disable(WebGlRenderingContext::BLEND);
    scene
        .reticle_prog
        .draw(&scene.gl, arrow_projection, [1., 0., 0.]);
    scene
        .reticle_prog
        .draw(&scene.gl, arrow_projection * rot_y, [0., 1., 0.]);
    scene
        .reticle_prog
        .draw(&scene.gl, arrow_projection * rot_z, [0., 0., 1.]);
}

/// Sets up legion ECS for debug info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup)
}
