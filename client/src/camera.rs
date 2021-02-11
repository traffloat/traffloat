use std::f64::consts::PI;
use std::sync::Mutex;

use crate::{config, input, render};
use traffloat::space::{Matrix, Point, Position, Vector};
use traffloat::time;

const DRAG_DEADZONE: u32 = 5;

mod unsafe_proj {
    use super::*;

    #[derive(Debug, getset::CopyGetters, typed_builder::TypedBuilder)]
    pub struct Camera {
        /// The point focused by the camera.
        ///
        /// `projection()` shall transform this point to (0.5, 0.5, z), where `z = zoom / distance`.
        #[getset(get_copy = "pub")]
        focus: Position,
        /// The rotation matrix of the camera.
        ///
        /// This transforms the real coordinates to the coordinates as seen by the user.
        #[getset(get_copy = "pub")]
        rotation: Matrix,

        /// Canvas width divided by canvas height
        #[getset(get_copy = "pub")]
        aspect: f64,
        /// The distance of the focus from the camera.
        #[getset(get_copy = "pub")]
        zoom: f64,
        /// The rendering distance of the camera.
        #[getset(get_copy = "pub")]
        distance: f64,
        /// The vertical field of view in radians.
        #[getset(get_copy = "pub")]
        fovy: f64,

        #[builder(default)]
        proj: Mutex<Option<Matrix>>,
        #[builder(default)]
        proj_inv: Mutex<Option<Matrix>>,
    }

    impl Camera {
        pub fn set_focus(&mut self, focus: Position) {
            self.focus = focus;
            self.invalidate();
        }

        pub fn set_rotation(&mut self, rotation: Matrix) {
            self.rotation = rotation;
            self.invalidate();
        }

        pub fn set_aspect(&mut self, aspect: f64) {
            self.aspect = aspect;
            self.invalidate();
        }

        pub fn set_zoom(&mut self, zoom: f64) {
            self.zoom = zoom;
            self.invalidate();
        }

        pub fn set_distance(&mut self, distance: f64) {
            self.distance = distance;
            self.invalidate();
        }

        pub fn set_fovy(&mut self, fovy: f64) {
            self.fovy = fovy;
            self.invalidate();
        }

        fn invalidate(&mut self) {
            *self.proj.get_mut().expect("Lock poisoned") = None;
            *self.proj_inv.get_mut().expect("Lock poisoned") = None;
        }

        /// Transforms real coordinates to unit cube [0, 1]^3
        pub fn projection(&self) -> Matrix {
            let mut proj = self.proj.lock().expect("Lock poisoned");
            *proj.get_or_insert_with(|| {
                let mut matrix = Matrix::identity();

                // Translate the focus to the origin
                matrix.append_translation_mut(&-self.focus.vector());

                // Rotate the world
                matrix = self.rotation * matrix;

                // Move backwards to the camera position
                matrix.append_translation_mut(&Vector::new(0., 0., self.zoom));

                // Finally, apply projection matrix
                matrix = Matrix::new_perspective(self.aspect, self.fovy, self.zoom, self.distance)
                    * matrix;

                matrix
            })
        }

        /// Transforms unit cube [0, 1]^2 to real coordinates
        pub fn inv_projection(&self) -> Matrix {
            let mut proj_inv = self.proj_inv.lock().expect("Lock poisoned");
            *proj_inv.get_or_insert_with(|| {
                self.projection()
                    .try_inverse()
                    .expect("Projection matrix is singular")
            })
        }
    }
}

pub use unsafe_proj::Camera;

impl Camera {
    /// Projects a mouse click from the unit square [0, 1]^2 to real coordinates.
    ///
    /// # Returns
    /// Returns a tuple `(a, b)`.
    /// The mouse clicks on points between points `a` and `b`.
    /// `a` is the closer point and `b` is the further point.
    pub fn project_mouse(&self, mut x: f64, mut y: f64) -> (Position, Position) {
        x = x * 2. - 1.;
        y = y * 2. - 1.;

        let mut a = Point::new(x, y, 0.);
        let mut b = Point::new(x, y, 1.);

        let matrix = self.inv_projection();
        a = matrix.transform_point(&a);
        b = matrix.transform_point(&b);
        (Position(a), Position(b))
    }
}

#[codegen::system]
#[allow(clippy::indexing_slicing, clippy::too_many_arguments)]
fn camera(
    #[resource] camera: &mut Camera,
    #[resource] actions: &input::keyboard::ActionSet,
    #[resource] clock: &time::Clock,
    #[resource] cursor_position: &input::mouse::CursorPosition,
    #[resource] dim: &render::Dimension,
    #[subscriber] wheel_events: impl Iterator<Item = input::mouse::WheelEvent>,
    #[state(None)] drag_start: &mut Option<(Position, (f64, f64))>,
    #[state(DRAG_DEADZONE)] drag_deadzone_count: &mut u32,
) {
    let dt = clock.delta.value() as f64;

    let mut move_direction = Vector::new(0., 0., 0.);
    if actions[input::keyboard::Action::Left] {
        move_direction += Vector::new(-config::WASD_VELOCITY * dt, 0., 0.);
    }
    if actions[input::keyboard::Action::Right] {
        move_direction += Vector::new(config::WASD_VELOCITY * dt, 0., 0.);
    }
    if actions[input::keyboard::Action::Down] {
        move_direction += Vector::new(0., -config::WASD_VELOCITY * dt, 0.);
    }
    if actions[input::keyboard::Action::Up] {
        move_direction += Vector::new(0., config::WASD_VELOCITY * dt, 0.);
    }
    if actions[input::keyboard::Action::Backward] {
        move_direction += Vector::new(0., 0., -config::WASD_VELOCITY * dt);
    }
    if actions[input::keyboard::Action::Forward] {
        move_direction += Vector::new(0., 0., config::WASD_VELOCITY * dt);
    }
    if move_direction != Vector::new(0., 0., 0.) {
        use nalgebra::dimension;

        let dp = camera.rotation().transform_vector(&move_direction);
        camera.set_focus(camera.focus() + dp);
    }

    if actions[input::keyboard::Action::ZoomIn] {
        camera.set_zoom(camera.zoom() - config::ZOOM_VELOCITY * dt);
    }
    if actions[input::keyboard::Action::ZoomOut] {
        camera.set_zoom(camera.zoom() + config::ZOOM_VELOCITY * dt);
    }

    for wheel in wheel_events {
        if wheel.delta > 0. {
            camera.set_zoom(camera.zoom() + config::SCROLL_VELOCITY);
        } else {
            camera.set_zoom(camera.zoom() - config::SCROLL_VELOCITY);
        }
    }

    #[allow(clippy::float_cmp)] // we simply want to see if it *might* have changed
    if camera.aspect() != dim.aspect() {
        camera.set_aspect(dim.aspect());
    }
}

pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup
        .resource(
            Camera::builder()
                .focus(Position::new(0., 0., 0.))
                .rotation(Matrix::identity())
                .aspect(1.)
                .zoom(0.)
                .distance(100.)
                .fovy(PI / 4.)
                .build(),
        )
        .uses(camera_setup)
}
