//! Renders nodes, edges and vehicles.

use std::f64::consts::PI;

use legion::world::SubWorld;
use legion::{component, Entity};
use safety::Safety;
use traffloat::units;
use web_sys::WebGlRenderingContext;

use super::{CursorType, RenderFlag};
use crate::camera::Camera;
use crate::{input, options};
use traffloat::appearance::{self, Appearance};
use traffloat::lerp;
use traffloat::space::{Matrix, Position, Vector};
use traffloat::sun::{LightStats, Sun, MONTH_COUNT};

pub mod mesh;

pub mod edge;
pub mod node;
pub mod reticle;

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

        Self { gl, node_prog, edge_prog, reticle_prog }
    }

    /// Clears the canvas.
    pub fn clear(&self) {
        self.gl.clear_color(0., 0., 0., 0.);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    }

    /// Sets the cursor icon.
    pub fn set_cursor(&self, name: &str) {
        use wasm_bindgen::JsCast;

        let canvas: web_sys::HtmlCanvasElement = self
            .gl
            .canvas()
            .expect("UI does not have a canvas")
            .dyn_into()
            .expect("Canvas is not a HtmlCanvasElement");
        canvas.style().set_property("cursor", name).expect("Failed to set canvas cursor property");
    }
}

#[codegen::system(Visualize)]
#[read_component(Position)]
#[read_component(appearance::Appearance)]
#[read_component(LightStats)]
#[read_component(units::Portion<units::Hitpoint>)]
#[read_component(traffloat::liquid::Storage)]
#[read_component(traffloat::liquid::StorageSize)]
#[read_component(traffloat::liquid::NextStorageSize)]
#[read_component(traffloat::node::Id)]
#[read_component(traffloat::edge::Id)]
#[read_component(traffloat::edge::Size)]
#[thread_local]
fn draw(
    world: &mut SubWorld,
    #[resource] camera: &Camera,
    #[resource] layers: &Option<super::Layers>,
    #[resource] sun: &Sun,
    #[resource] texture_pool: &mut Option<super::texture::Pool>,
    #[resource] hover_target: &input::mouse::HoverTarget,
    #[resource] focus_target: &input::FocusTarget,
    #[resource(no_init)] options: &options::Options,
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

    let texture_pool = texture_pool.get_or_insert_with(|| super::texture::Pool::new(&scene.gl));

    let sun_dir = sun.direction();

    scene.gl.enable(WebGlRenderingContext::CULL_FACE);
    scene.gl.enable(WebGlRenderingContext::BLEND);

    type Comps<'t> =
        (Entity, &'t Position, &'t Appearance, &'t LightStats, &'t units::Portion<units::Hitpoint>);
    let base_month: f64 = sun.yaw() / PI / 2. * MONTH_COUNT.small_float::<f64>();
    let mut scales: [Box<dyn options::ColorMapCount<Comps<'_>>>; 2] = [
        Box::new(options::ColorMapCounter::try_new(
            options.graphics().node().brightness(),
            |(_, _, _, light, _hp): Comps| {
                #[allow(clippy::indexing_slicing, clippy::cast_possible_truncation)]
                let prev = light.brightness()[base_month.floor() as usize % MONTH_COUNT];
                #[allow(clippy::indexing_slicing, clippy::cast_possible_truncation)]
                let next = light.brightness()[base_month.ceil() as usize % MONTH_COUNT];
                let lerped = lerp(prev, next, base_month.fract());
                lerped.value()
            },
        )),
        Box::new(options::ColorMapCounter::try_new(
            options.graphics().node().hitpoint(),
            |(_, _, _, _light, hp): Comps| hp.ratio(),
        )),
    ];

    if scales.iter().any(|scale| scale.is_some()) {
        for comps in Comps::query().filter(component::<traffloat::node::Id>()).iter(world) {
            let comps = (*comps.0, comps.1, comps.2, comps.3, comps.4); // hack, blame bad legion design :(

            for scale in &mut scales {
                scale.feed(comps);
            }
        }
    }

    // Draw nodes
    if options.graphics().node().render() {
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        for comps in Comps::query().filter(component::<traffloat::node::Id>()).iter(world) {
            let comps = (*comps.0, comps.1, comps.2, comps.3, comps.4); // hack, blame bad legion design :(

            let mut filter = Vector::new(1., 1., 1.);

            for scale in &scales {
                let subfilter = scale.compute(comps);
                filter.component_mul_assign(&subfilter);
            }

            let (entity, position, appearance, _, _) = comps;

            let selected =
                hover_target.entity() == Some(entity) || focus_target.entity() == Some(entity);

            for component in appearance.components() {
                // projection matrix transforms real coordinates to canvas

                let unit_to_real = component.transform(*position);
                let tex: &appearance::Texture = component.texture();
                let sprite = texture_pool.sprite(tex, &scene.gl);

                scene.node_prog.draw(
                    node::DrawArgs::builder()
                        .gl(&scene.gl)
                        .proj(projection * unit_to_real)
                        .sun(sun_dir)
                        .filter(filter)
                        .selected(selected)
                        .texture(&sprite)
                        .shape_unit(component.unit())
                        .build(),
                );
            }
        }
    }

    // Draw edges
    if options.graphics().edge().render() {
        for (entity, edge, size) in
            <(Entity, &traffloat::edge::Id, &traffloat::edge::Size)>::query().iter(world)
        {
            let unit = traffloat::edge::tf(edge, size, &*world, true);
            let selected =
                hover_target.entity() == Some(*entity) || focus_target.entity() == Some(*entity);

            let rgb = options.graphics().edge().base();
            let rgba = rgb.fixed_resize::<4, 1>(1.);
            scene.edge_prog.draw(
                &scene.gl,
                projection * unit,
                projection.transform_vector(&sun_dir),
                rgba,
                selected,
                options.graphics().edge().reflection(),
            );
        }
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
    if options.graphics().render_reticle() {
        scene.reticle_prog.draw(&scene.gl, arrow_projection, [1., 0., 0.]);
        scene.reticle_prog.draw(&scene.gl, arrow_projection * rot_y, [0., 1., 0.]);
        scene.reticle_prog.draw(&scene.gl, arrow_projection * rot_z, [0., 0., 1.]);
    }
}

#[codegen::system(Visualize)]
#[thread_local]
fn update_cursor(
    #[resource] canvas: &Option<super::Layers>,
    #[resource] cursor_type: &CursorType,
    #[subscriber] render_flag: impl Iterator<Item = RenderFlag>,
) {
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
    scene.set_cursor(cursor_type.name());
}

/// Sets up legion ECS for debug info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup).uses(update_cursor_setup)
}
