use specs::Join;

use common::shape::{self, Shape};
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
        specs::Read<'a, Clock>,
    );

    fn run(&mut self, (camera, canvas, rendered, shapes, clock): Self::SystemData) {
        let canvas = match canvas {
            Some(canvas) => canvas,
            None => return,
        };

        if !canvas.render_requested {
            return;
        }

        {
            let elapsed = clock.now.0;
            let noise = elapsed.0 as i32 % 10_i32;
            canvas.render_bg(camera.star_matrix(noise));
        }

        let camera_matrix = camera.inv_transform();

        canvas.render_shape(
            camera_matrix,
            Shape {
                unit: shape::Unit::Tetra,
                transform: Matrix::identity()
                    .append_scaling(0.1)
                    .append_translation(&Vector::new(0., 0., -1.)),
            },
        );
        canvas.render_shape(
            camera_matrix,
            Shape {
                unit: shape::Unit::Sphere,
                transform: Matrix::identity()
                    .append_scaling(0.1)
                    .append_translation(&Vector::new(0., -0.5, -1.)),
            },
            );
        canvas.render_shape(
            camera_matrix,
            Shape {
                unit: shape::Unit::Cube,
                transform: Matrix::identity()
                    .append_scaling(0.1)
                    .append_translation(&Vector::new(0., 0.5, -1.)),
            },
            );

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
