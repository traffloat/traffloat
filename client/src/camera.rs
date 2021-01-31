use crate::{config, input, render};
use traffloat::types::{Clock, Matrix, Point, Position, Vector};

#[derive(Debug)]
pub struct Camera {
    /// Center position of the camera
    pub position: Vector,
    /// The screen height rendered
    pub render_height: f64,
}

impl Camera {
    /// The area viewed by the camera
    pub fn viewport(&self, aspect: f64) -> (Vector, Vector) {
        // aspect = canvas.width / canvas.height
        // viewport.height / canvas.height = viewport.width / canvas.width
        // viewport.width = viewport.height * aspect
        let render_width = self.render_height * aspect;

        let semidiagonal = Vector::new(render_width, self.render_height) / 2.;
        (self.position - semidiagonal, self.position + semidiagonal)
    }

    /// Projects physical coordinates to canvas coordinates
    pub fn projection(&self, dim: render::Dimension) -> Matrix {
        // project viewport to (origin, dim)
        let (viewport_start, viewport_end) = self.viewport(dim.aspect());
        let viewport_size = viewport_end - viewport_start;

        let canvas = dim.as_vector();

        let mut ret = Matrix::identity();

        // translate viewport center as origin
        ret.append_translation_mut(&-self.position);

        // scale from viewport size to canvas size
        let scaling = canvas.component_div(&viewport_size);
        ret.append_nonuniform_scaling_mut(&scaling);

        // translate origin as canvas center
        ret.append_translation_mut(&(canvas / 2.));

        ret
    }

    /// Converts coordinates in the unit square with inversed y to real coordinates
    pub fn image_unit_to_real(&self, mut image: Vector, aspect: f64) -> Position {
        // correct y axis
        #[allow(clippy::indexing_slicing)]
        {
            image[1] = 1. - image[1];
        }

        let (viewport_start, viewport_end) = self.viewport(aspect);
        let viewport_size = viewport_end - viewport_start;

        let transform = Matrix::identity()
            // translate unit square center to origin
            .append_translation(&Vector::new(-0.5, -0.5))
            // scale from unit size to viewport size
            .append_nonuniform_scaling(&viewport_size)
            // translate origin to viewport center
            .append_translation(&self.position);

        #[allow(clippy::indexing_slicing)]
        {
            Position(transform.transform_point(&Point::new(image[0], image[1])))
        }
    }
}

#[legion::system]
#[allow(clippy::indexing_slicing)]
fn camera(
    #[resource] camera: &mut Camera,
    #[resource] actions: &mut input::keyboard::ActionSet,
    #[resource] clock: &mut Clock,
) {
    if actions[input::keyboard::Action::Left] {
        camera.position -=
            Vector::new(1., 0.) * config::WASD_VELOCITY * (clock.delta.value() as f64);
    }
    if actions[input::keyboard::Action::Right] {
        camera.position +=
            Vector::new(1., 0.) * config::WASD_VELOCITY * (clock.delta.value() as f64);
    }
    if actions[input::keyboard::Action::Down] {
        camera.position -=
            Vector::new(0., 1.) * config::WASD_VELOCITY * (clock.delta.value() as f64);
    }
    if actions[input::keyboard::Action::Up] {
        camera.position +=
            Vector::new(0., 1.) * config::WASD_VELOCITY * (clock.delta.value() as f64);
    }

    if actions[input::keyboard::Action::ZoomIn] {
        camera.render_height /= config::ZOOM_RATE.powi(clock.delta.value() as i32);
    }
    if actions[input::keyboard::Action::ZoomOut] {
        camera.render_height *= config::ZOOM_RATE.powi(clock.delta.value() as i32);
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
