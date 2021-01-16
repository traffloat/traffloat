use std::f32::consts::PI;

use crate::config;
use crate::keymap::{self, Action};
use common::types::*;

pub struct Camera {
    pub pos: Vector,
    pub zoom: Vector,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn inv_transform(&self) -> Matrix {
        let mut matrix = Matrix::identity();
        matrix.append_translation_mut(&-self.pos);
        matrix.append_nonuniform_scaling_mut(&self.zoom);
        matrix = Matrix::from_euler_angles(-self.roll, -self.pitch, -self.yaw) * matrix;
        Matrix::new_perspective(self.aspect, self.fovy, self.znear, self.zfar) * matrix
    }

    pub fn star_matrix(&self) -> Matrix {
        let rot = Matrix::from_euler_angles(-self.roll, -self.pitch, -self.yaw);
        Matrix::new_perspective(self.aspect, self.fovy, 0.9, 1.1) * rot
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: Vector::new(0., 0., 5.),
            zoom: Vector::new(1., 1., 1.) * 0.1,
            yaw: PI,
            pitch: 0.,
            roll: 0.,
            aspect: 1.,
            fovy: PI / 4.,
            znear: 0.1,
            zfar: 1.,
        }
    }
}

pub struct ViewSystem(());

impl ViewSystem {
    pub fn new(world: &mut specs::World) -> Self {
        use specs::SystemData;

        <Self as specs::System<'_>>::SystemData::setup(world);
        Self(())
    }
}

impl<'a> specs::System<'a> for ViewSystem {
    type SystemData = (
        specs::Read<'a, keymap::CurrentActions>,
        specs::Read<'a, Clock>,
        specs::Write<'a, Camera>,
    );

    fn run(&mut self, (action_set, clock, mut camera): Self::SystemData) {
        let time_delta = clock.delta.as_secs();

        log::debug!("Actions: {:?}", action_set.actions().collect::<Vec<_>>());

        for action in action_set.actions() {
            match action {
                Action::MoveDown => {
                    camera.pos -= Vector::new(0., config::MOVE_SPEED * time_delta, 0.);
                }
                Action::MoveUp => {
                    camera.pos += Vector::new(0., config::MOVE_SPEED * time_delta, 0.);
                }
                Action::MoveLeft => {
                    camera.pos -= Vector::new(config::MOVE_SPEED * time_delta, 0., 0.);
                }
                Action::MoveRight => {
                    camera.pos += Vector::new(config::MOVE_SPEED * time_delta, 0., 0.);
                }
                Action::MoveBack => {
                    camera.pos -= Vector::new(0., 0., config::MOVE_SPEED * time_delta);
                }
                Action::MoveFront => {
                    camera.pos += Vector::new(0., 0., config::MOVE_SPEED * time_delta);
                }
                Action::RotDown => {
                    camera.pitch -= config::ROT_PITCH_SPEED * time_delta;
                }
                Action::RotUp => {
                    camera.pitch += config::ROT_PITCH_SPEED * time_delta;
                }
                Action::RotLeft => {
                    camera.yaw -= config::ROT_YAW_SPEED * time_delta;
                }
                Action::RotRight => {
                    camera.yaw += config::ROT_YAW_SPEED * time_delta;
                }
                _ => {}
            }
        }
    }
}
