//! Provides a camera resource to store the view perspective of the user.

use std::f64::consts::PI;
use std::sync::Mutex;

use legion::Entity;

use crate::config;
use crate::input::keyboard;
use crate::render;

use traffloat::space::{Matrix, Point, Position, Vector};
use traffloat::time;

/// Visibiilty guard to avoid inconsistent `proj` updates.
mod unsafe_proj {
    use super::*;

    /// A resource that stores the view perspective of the user.
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
        /// Sets the point focused by the camera.
        ///
        /// The `focus` point is transformed to (0.5, 0.5, `zoom/distance`) by the projection
        /// matrix.
        pub fn set_focus(&mut self, focus: Position) {
            self.focus = focus;
            self.invalidate();
        }

        /// Sets the rotation matrix of the camera.
        ///
        /// This transforms the real coordinates to the coordinates as seen by the user.
        pub fn set_rotation(&mut self, rotation: Matrix) {
            self.rotation = rotation;
            self.invalidate();
        }

        /// Sets the aspect ratio, which is the canvas width divided by the canvas height.
        pub fn set_aspect(&mut self, aspect: f64) {
            self.aspect = aspect;
            self.invalidate();
        }

        /// Sets the distance of `focus` from the camera.
        pub fn set_zoom(&mut self, zoom: f64) {
            self.zoom = zoom;
            self.invalidate();
        }

        /// Sets the rendering distance of the camera.
        pub fn set_distance(&mut self, distance: f64) {
            self.distance = distance;
            self.invalidate();
        }

        /// Sets the vertical Field of View in radians.
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
                matrix =
                    Matrix::new_perspective(self.aspect, 1.5, self.zoom, self.distance) * matrix;

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

impl Default for Camera {
    fn default() -> Self {
        Camera::builder()
            .focus(Position::new(1.5, 2., 6.))
            .rotation(Matrix::identity())
            .aspect(1.)
            .zoom(0.01)
            .distance(100.)
            .fovy(PI / 4.)
            .build()
    }
}

/// A resource storing the current cursor-pointed target.
#[derive(getset::CopyGetters, Default)]
pub struct CursorTarget {
    /// The line segment from the closest point to the furthest point under the cursor.
    #[getset(get_copy = "pub")]
    segment: Option<(Position, Position)>,

    /// The entity pointed by the cursor.
    #[getset(get_copy = "pub")]
    entity: Option<Entity>,
}

#[codegen::system]
#[allow(clippy::indexing_slicing, clippy::too_many_arguments)]
fn keyboard(
    #[resource] camera: &mut Camera,
    #[resource] clock: &time::Clock,
    #[resource] commands: &keyboard::CommandStates,
    #[resource(no_init)] dim: &render::Dimension,
) {
    let dt = clock.delta().value() as f64;

    if commands[keyboard::Command::RotationMask].active() {
        let mut roll = 0.;
        let mut pitch = 0.;
        let mut yaw = 0.;
        if commands[keyboard::Command::MoveLeft].active() {
            yaw -= config::WASD_ROTATION_VELOCITY;
        }
        if commands[keyboard::Command::MoveRight].active() {
            yaw += config::WASD_ROTATION_VELOCITY;
        }
        if commands[keyboard::Command::MoveDown].active() {
            pitch += config::WASD_ROTATION_VELOCITY;
        }
        if commands[keyboard::Command::MoveUp].active() {
            pitch -= config::WASD_ROTATION_VELOCITY;
        }
        if commands[keyboard::Command::MoveFront].active() {
            roll -= config::WASD_ROTATION_VELOCITY;
        }
        if commands[keyboard::Command::MoveBack].active() {
            roll += config::WASD_ROTATION_VELOCITY;
        }
        if roll != 0. || pitch != 0. || yaw != 0. {
            let mat = nalgebra::Rotation3::from_euler_angles(pitch, yaw, roll).to_homogeneous();
            camera.set_rotation(mat * camera.rotation());
        }
    } else {
        let mut move_direction = Vector::new(0., 0., 0.);
        if commands[keyboard::Command::MoveLeft].active() {
            move_direction += Vector::new(-config::WASD_LINEAR_VELOCITY * dt, 0., 0.);
        }
        if commands[keyboard::Command::MoveRight].active() {
            move_direction += Vector::new(config::WASD_LINEAR_VELOCITY * dt, 0., 0.);
        }
        if commands[keyboard::Command::MoveUp].active() {
            move_direction += Vector::new(0., config::WASD_LINEAR_VELOCITY * dt, 0.);
        }
        if commands[keyboard::Command::MoveDown].active() {
            move_direction += Vector::new(0., -config::WASD_LINEAR_VELOCITY * dt, 0.);
        }
        if commands[keyboard::Command::MoveFront].active() {
            move_direction += Vector::new(0., 0., -config::WASD_LINEAR_VELOCITY * dt);
        }
        if commands[keyboard::Command::MoveBack].active() {
            move_direction += Vector::new(0., 0., config::WASD_LINEAR_VELOCITY * dt);
        }
        if move_direction != Vector::new(0., 0., 0.) {
            let dp = camera
                .rotation()
                .try_inverse()
                .expect("Rotation matrix is singular")
                .transform_vector(&move_direction);
            camera.set_focus(camera.focus() + dp);
        }
    }
    if commands[keyboard::Command::ZoomIn].active() {
        camera.set_zoom(camera.zoom() + config::ZOOM_VELOCITY * dt);
    }
    if commands[keyboard::Command::ZoomOut].active() {
        // camera.set_zoom(f64::max(camera.zoom() - config::ZOOM_VELOCITY * dt, 0.0001));
        camera.set_zoom(camera.zoom() - config::ZOOM_VELOCITY * dt);
    }

    /*
    for wheel in wheel_events {
        if wheel.delta > 0. {
            camera.set_zoom(camera.zoom() + config::SCROLL_VELOCITY);
        } else {
            camera.set_zoom(camera.zoom() - config::SCROLL_VELOCITY);
        }
    }
    */

    #[allow(clippy::float_cmp)] // we simply want to see if it *might* have changed
    if camera.aspect() != dim.aspect() {
        camera.set_aspect(dim.aspect());
    }

    /*
    for event in drag_events {
        let (action, prev, now) = match event {
            input::mouse::DragEvent::Move {
                action, prev, now, ..
            } => (action, prev, now),
            _ => continue,
        };

        let delta = *now - *prev;
        match action {
            input::keyboard::Action::LeftClick => {
                // TODO rotation
            }
            input::keyboard::Action::RightClick => {
                // TODO motion
            }
            _ => {} // unused
        }
    }
    */
}

/*
#[codegen::system]
#[read_component(Shape)]
#[read_component(Position)]
#[read_component(input::mouse::Clickable)]
fn locate_cursor(
    world: &legion::world::SubWorld,
    #[resource] cursor: &input::mouse::CursorPosition,
    #[resource] target: &mut CursorTarget,
) {
    /*
    TODO
    cursor.entity = Err((x, y));
    comm.canvas_cursor_type.set("initial");
    for (entity, &position, shape, clickable) in
        <(Entity, &Position, &Shape, &Clickable)>::query().iter(world)
    {
        if !clickable.0 { continue; }

        let point = shape
            .transform(position)
            .try_inverse()
            .expect("Transformation matrix is singular")
            .transform_point(&real_pos.0);
        if shape.unit.contains(point) {
            cursor.entity = Ok(*entity);
            comm.canvas_cursor_type.set("pointer");
            break;
        }
    }
    */
}
*/

/// Sets up legion ECS for this module.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(keyboard_setup)
}
