//! Manages client-side graphics rendering.

use std::f64::consts::PI;

use crate::camera::Camera;
use crate::util::lerp;
use codegen::hrtime;
use safety::Safety;
use traffloat::config;
use traffloat::shape::{Shape, Texture};
use traffloat::space::{Matrix, Position, Vector};
use traffloat::sun::{LightStats, Sun, MONTH_COUNT};
use traffloat::time;

mod canvas;
pub use canvas::*;
mod comm;
pub use comm::*;
mod image;
pub use image::*;

pub mod bg;
pub mod debug;
pub mod scene;
pub mod ui;

pub use scene::Renderable;

mod util;

/// Sets up legion ECS for rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup
        .uses(bg::setup_ecs)
        .uses(scene::setup_ecs)
        .uses(debug::setup_ecs)
}
