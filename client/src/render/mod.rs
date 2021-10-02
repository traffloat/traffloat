//! Manages client-side graphics rendering.

// #[cfg(debug_assertions)]
macro_rules! glsl {
    ($name:literal) => {
        [
            concat!($name, ".vert"),
            include_str!(concat!($name, ".vert")),
            concat!($name, ".frag"),
            include_str!(concat!($name, ".frag")),
        ]
    };
}

// #[cfg(not(debug_assertions))]
// macro_rules! glsl {
//     ($name:literal) => {
//         [
//             concat!($name, ".vert"),
//             include_str!(concat!($name, ".min.vert")),
//             concat!($name, ".frag"),
//             include_str!(concat!($name, ".min.frag")),
//         ]
//     };
// }

mod layers;
pub use layers::{Dimension, Layers, LayersStruct};
mod comm;
pub use comm::*;

pub mod texture;

pub mod bg;
pub mod debug;
pub mod scene;
pub mod ui;

pub mod util;

/// Sets up legion ECS for rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(bg::setup_ecs).uses(scene::setup_ecs).uses(ui::setup_ecs).uses(debug::setup_ecs)
}
