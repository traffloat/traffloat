//! Provides a camera resource to store the view perspective of the user.

use std::f64::consts::PI;
use std::sync::Mutex;

use traffloat::space::{LinearMatrix, Matrix, Point, Position, Vector};

/// Rightward axis for clip space.
pub const GL_RIGHT_DIR: Vector = Vector::new(1., 0., 0.);
/// Upward axis for clip space.
pub const GL_TOP_DIR: Vector = Vector::new(0., 1., 0.);
/// Forward axis for clip space.
pub const GL_VIEW_DIR: Vector = Vector::new(0., 0., 1.);

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

                // Now the target is in view space.
                // Move backwards to the camera position
                matrix.append_translation_mut(&Vector::new(0., 0., self.zoom));

                // Finally, apply projection matrix
                matrix = self.perspective() * matrix;

                matrix
            })
        }

        /// Transforms unit cube [0, 1]^2 to real coordinates
        pub fn inv_projection(&self) -> Matrix {
            let mut proj_inv = self.proj_inv.lock().expect("Lock poisoned");
            *proj_inv.get_or_insert_with(|| {
                self.projection().try_inverse().expect("Projection matrix is singular")
            })
        }

        /// An "asymptotic" version of the projection matrix, which does not perform translation.
        pub fn asymptotic_projection(&self) -> LinearMatrix {
            self.projection().fixed_slice::<3, 3>(0, 0).into()
        }

        /// Compute the perspective matrix transforming view space to clip space
        fn perspective(&self) -> Matrix {
            let znear = 0.1;
            let zfar = self.distance + self.zoom; // furthest point offset by zoom.
            Matrix::new_perspective(self.aspect, self.fovy, znear, zfar)
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

        let focus = Point::origin() + GL_RIGHT_DIR * x + GL_TOP_DIR * -y;

        let matrix = self.inv_projection();
        let proximal = matrix.transform_point(&focus);
        let distal = matrix.transform_point(&(focus + GL_VIEW_DIR));
        (Position(proximal), Position(distal))
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera::builder()
            .focus(Position::new(0., 0., 16.))
            .rotation(Matrix::identity())
            .aspect(1.)
            .zoom(-10.)
            .distance(500.)
            .fovy(PI / 4.)
            .build()
    }
}

#[codegen::system]
fn debug(
    #[resource] camera: &Camera,

    #[debug("Camera", "Position")] focus_debug: &codegen::DebugEntry,
    #[debug("Camera", "Facing")] dir_debug: &codegen::DebugEntry,
    #[debug("Camera", "Render distance")] dist_debug: &codegen::DebugEntry,
    #[debug("Camera", "Zoom")] zoom_debug: &codegen::DebugEntry,
) {
    codegen::update_debug!(
        focus_debug,
        "({:.1}, {:.1}, {:.1})",
        camera.focus().x(),
        camera.focus().y(),
        camera.focus().z()
    );

    let line_of_sight = camera.rotation().transpose().transform_vector(&Vector::new(0., 0., -1.));
    codegen::update_debug!(
        dir_debug,
        "({:.1}, {:.1}, {:.1})",
        line_of_sight.x,
        line_of_sight.y,
        line_of_sight.z,
    );

    codegen::update_debug!(dist_debug, "{}", camera.distance());
    codegen::update_debug!(zoom_debug, "{}", camera.zoom());
}

/// Sets up legion ECS for this module.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(debug_setup)
}
