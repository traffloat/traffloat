//! Manages client-side graphics rendering.

mod canvas;
pub use canvas::*;
mod comm;
pub use comm::*;

mod texture;
pub use texture::*;

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
        .uses(ui::setup_ecs)
        .uses(debug::setup_ecs)
}
