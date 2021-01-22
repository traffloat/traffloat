use std::f32::consts::PI;

use lazy_static::lazy_static;

use crate::config;
use crate::keymap::{self, Action};
use common::types::*;

#[derive(Debug, Component)]
#[storage(storage::BTreeStorage)]
pub struct Camera {
    pub pos: Vector,
    pub zoom: f32,
    pub rot: Matrix,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn inv_transform(&self) -> Matrix {
        let trans = Matrix::identity().append_translation(&-self.pos);
        let zoom = Matrix::identity().append_scaling(self.zoom);
        let rot = self.rot_matrix(0);
        let pers = Matrix::new_perspective(self.aspect, self.fovy, 0.1, 100.);
        pers * zoom * rot * trans
    }

    pub fn star_matrix(&self, noise: i32) -> Matrix {
        let zoom = Matrix::identity().append_scaling(self.zoom);
        let rot = self.rot_matrix(noise);
        let pers = Matrix::new_perspective(self.aspect, self.fovy, 0., 1.01);
        pers * rot * zoom
    }

    #[inline(always)]
    fn rot_matrix(&self, noise: i32) -> Matrix {
        lazy_static! {
            static ref NOISE_MATRIX: Matrix =
                Matrix::from_euler_angles(0., 0., config::BG_STAR_NOISE);
        }

        let mut ret = self.rot;
        if noise % 2 == 1 {
            ret *= *NOISE_MATRIX;
        }
        ret
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: Vector::new(0., 0., 0.),
            zoom: 1.,
            rot: Matrix::identity(),
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

        lazy_static! {
            static ref YAW_DEC: Matrix =
                Matrix::from_euler_angles(0., -config::ROT_PITCH_SPEED * 0.01, 0.);
            static ref YAW_INC: Matrix =
                Matrix::from_euler_angles(0., config::ROT_PITCH_SPEED * 0.01, 0.);
            static ref PITCH_DEC: Matrix =
                Matrix::from_euler_angles(config::ROT_YAW_SPEED * 0.01, 0., 0.);
            static ref PITCH_INC: Matrix =
                Matrix::from_euler_angles(-config::ROT_YAW_SPEED * 0.01, 0., 0.);
            static ref ROLL_DEC: Matrix =
                Matrix::from_euler_angles(0., 0., config::ROT_ROLL_SPEED * 0.01);
            static ref ROLL_INC: Matrix =
                Matrix::from_euler_angles(0., 0., -config::ROT_ROLL_SPEED * 0.01);
        }

        use nalgebra::dimension as dim;
        type Matrix3 = nalgebra::Matrix3<f32>;
        let rot = camera
            .rot
            .fixed_slice::<dim::U3, dim::U3>(0, 0)
            .try_inverse()
            .unwrap_or_else(Matrix3::identity);

        for action in action_set.actions() {
            match action {
                Action::MoveDown => {
                    camera.pos -= rot * Vector::new(0., config::MOVE_SPEED * time_delta, 0.);
                }
                Action::MoveUp => {
                    camera.pos += rot * Vector::new(0., config::MOVE_SPEED * time_delta, 0.);
                }
                Action::MoveLeft => {
                    camera.pos -= rot * Vector::new(config::MOVE_SPEED * time_delta, 0., 0.);
                }
                Action::MoveRight => {
                    camera.pos += rot * Vector::new(config::MOVE_SPEED * time_delta, 0., 0.);
                }
                Action::MoveBack => {
                    camera.pos += rot * Vector::new(0., 0., config::MOVE_SPEED * time_delta);
                }
                Action::MoveFront => {
                    camera.pos -= rot * Vector::new(0., 0., config::MOVE_SPEED * time_delta);
                }
                Action::RotDown => {
                    camera.rot = *PITCH_DEC * camera.rot;
                }
                Action::RotUp => {
                    camera.rot = *PITCH_INC * camera.rot;
                }
                Action::RotLeft => {
                    camera.rot = *YAW_DEC * camera.rot;
                }
                Action::RotRight => {
                    camera.rot = *YAW_INC * camera.rot;
                }
                Action::RotAntiClock => {
                    camera.rot = *ROLL_DEC * camera.rot;
                }
                Action::RotClock => {
                    camera.rot = *ROLL_INC * camera.rot;
                }
                Action::ZoomIn => {
                    camera.zoom *= config::ZOOM_SPEED.powf(time_delta);
                }
                Action::ZoomOut => {
                    camera.zoom /= config::ZOOM_SPEED.powf(time_delta);
                }
                _ => {}
            }
        }
    }
}
