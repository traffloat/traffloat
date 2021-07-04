//! Manages client-side graphics rendering.

mod layers;
pub use layers::{Dimension, Layers, LayersStruct};
mod comm;
pub use comm::*;

pub mod bg;
pub mod debug;
pub mod scene;
pub mod ui;

mod util;

/// Sets up legion ECS for rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup
        .uses(bg::setup_ecs)
        .uses(scene::setup_ecs)
        .uses(ui::setup_ecs)
        .uses(debug::setup_ecs)
}
