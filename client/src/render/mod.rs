use specs::Join;

use common::heat::Sun;
use common::shape::{self, Shape};
use common::types::*;

mod camera;
pub use camera::Camera;

mod canvas;
pub use canvas::RenderContext;

#[derive(Debug, Component, Default)]
#[storage(storage::NullStorage)]
pub struct Rendered;

pub struct RenderSystem(());

impl RenderSystem {
    pub fn new(world: &mut specs::World) -> Self {
        use specs::SystemData;

        <Self as specs::System<'_>>::SystemData::setup(world);
        Self(())
    }
}

impl<'a> specs::System<'a> for RenderSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        specs::ReadExpect<'a, Camera>,
        WriteStorage<'a, RenderContext>,
        ReadStorage<'a, Rendered>,
        ReadStorage<'a, Shape>,
        specs::Read<'a, Clock>,
        specs::ReadExpect<'a, Sun>,
    );

    fn run(&mut self, (camera, mut canvas_store, rendered, shapes, clock, sun): Self::SystemData) {
        for canvas in (&mut canvas_store ).join() {
            if !canvas.should_render {
                return;
            }
            canvas.should_render = false;


            let camera_matrix = camera.inv_transform();

            for (_, shape) in (&rendered, &shapes).join() {
                if shape.is_clipped() {
                    // canvas.render_shape(camera_matrix, shape.clone(), sun.position, camera.pos);
                }
            }
        }
    }
}

pub fn setup_specs((mut world, mut dispatcher): common::Setup) -> common::Setup {
    use specs::WorldExt;

    world.register::<Rendered>();
    dispatcher = dispatcher.with_thread_local(RenderSystem::new(&mut world));
    dispatcher = dispatcher.with(
        camera::ViewSystem::new(&mut world),
        "camera_view",
        &["keymap"],
    );
    (world, dispatcher)
}
