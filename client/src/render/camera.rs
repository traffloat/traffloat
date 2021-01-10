use std::f32::consts::PI;

use super::{Matrix, Vector};

pub struct Camera {
    pub pos: Vector,
    pub zoom: Vector,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

impl Camera {
    pub fn inv_transform(&self) -> Matrix {
        /*
        let mut matrix = Matrix::identity();
        matrix.append_translation_mut(&-self.pos);
        matrix.append_nonuniform_scaling_mut(&self.zoom);
        // matrix = Matrix::from_euler_angles(-self.roll, -self.pitch, -self.yaw) * matrix;
        matrix
        */
        Matrix::new_perspective(1., PI / 4., 0.1, 100.)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: Vector::new(0., 0., 2.),
            zoom: Vector::new(1., 1., 1.) * 0.1,
            yaw: PI,
            pitch: 0.,
            roll: 0.,
        }
    }
}
