use traffloat::types::{Clock, Vector};
use crate::{config, input};

pub struct Camera {
    pub position: Vector,
    pub render_height: f32,
}

#[legion::system]
#[allow(clippy::indexing_slicing)]
fn camera(#[resource] camera: &mut Camera,
          #[resource] actions: &mut input::keyboard::ActionSet,
          #[resource] clock: &mut Clock) {
    if actions[input::keyboard::Action::Left] {
        camera.position -= Vector::new(1., 0.) * config::WASD_VELOCITY * (clock.delta.value() as f32);
    }
    if actions[input::keyboard::Action::Right] {
        camera.position += Vector::new(1., 0.) * config::WASD_VELOCITY * (clock.delta.value() as f32);
    }
    if actions[input::keyboard::Action::Up] {
        camera.position -= Vector::new(0., 1.) * config::WASD_VELOCITY * (clock.delta.value() as f32);
    }
    if actions[input::keyboard::Action::Down] {
        camera.position += Vector::new(0., 1.) * config::WASD_VELOCITY * (clock.delta.value() as f32);
    }

    if actions[input::keyboard::Action::ZoomIn] {
        camera.render_height *= config::ZOOM_RATE.powi(clock.delta.value() as i32);
    }

    if actions[input::keyboard::Action::ZoomIn] {
        camera.render_height /= config::ZOOM_RATE.powi(clock.delta.value() as i32);
    }
}

pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup
        .resource(Camera {
            position: Vector::new(0., 0.),
            render_height: 20.,
        })
        .system(camera_system())
}
