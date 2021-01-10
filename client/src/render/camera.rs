use std::f32::consts::PI;

use super::{Matrix, Vector};

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
