use specs::Join;

use common::shape::Shape;
use common::types::*;

mod camera;
pub use camera::Camera;

mod canvas;
pub use canvas::Canvas;

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
    type SystemData = (
        specs::ReadExpect<'a, Camera>,
        Option<specs::Read<'a, Canvas>>,
        ReadStorage<'a, Rendered>,
        ReadStorage<'a, Shape>,
    );

    fn run(&mut self, (camera, canvas, rendered, shapes): Self::SystemData) {
        let canvas = match canvas {
            Some(canvas) => canvas,
            None => return,
        };

        if !canvas.render_requested {
            return;
        }

        canvas.render_bg(camera.star_matrix());

        let camera_matrix = camera.inv_transform();
        for (_, shape) in (&rendered, &shapes).join() {
            if shape.is_clipped() {
                canvas.render_shape(camera_matrix, shape.clone());
            }
        }
    }
}

pub fn setup_specs((mut world, mut dispatcher): common::Setup) -> common::Setup {
    use specs::WorldExt;

    world.register::<Rendered>();
    dispatcher = dispatcher.with(RenderSystem::new(&mut world), "render", &[]);
    dispatcher = dispatcher.with(
        camera::ViewSystem::new(&mut world),
        "camera_view",
        &["keymap"],
    );
    (world, dispatcher)
}
