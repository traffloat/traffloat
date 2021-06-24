//! The webassembly client crate.

#![recursion_limit = "512"]
#![deny(
    anonymous_parameters,
    bare_trait_objects,
    clippy::clone_on_ref_ptr,
    clippy::float_cmp_const,
    clippy::if_not_else,
    clippy::unwrap_used
)]
#![cfg_attr(
    debug_assertions,
    allow(
        dead_code,
        unused_imports,
        unused_variables,
        clippy::match_single_binding,
    )
)]
#![cfg_attr(any(doc, not(debug_assertions)), warn(missing_docs))]
#![cfg_attr(
    not(debug_assertions),
    deny(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::dbg_macro,
        clippy::indexing_slicing,
    )
)]
#![feature(vecdeque_binary_search)]

use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod app;
pub mod camera;
pub mod config;
pub mod input;
pub mod render;
pub mod util;

/// Entry point.
#[wasm_bindgen(start)]
pub fn run_app() {
    std::panic::set_hook(Box::new(|info| {
        util::error_handler(&info.to_string());
    }));

    {
        let config = wasm_logger::Config::new(if cfg!(debug_assertions) {
            log::Level::Trace
        } else {
            log::Level::Info
        });
        wasm_logger::init(config);
    }

    App::<app::Mux>::new().mount_to_body();
}

/// Sets up legion ECS.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    use traffloat::space::{Matrix, Position};

    let setup = setup
        .uses(traffloat::setup_ecs)
        .uses(camera::setup_ecs)
        .uses(input::setup_ecs)
        .uses(render::setup_ecs);

    use traffloat::{
        config,
        shape::{self, Shape, Texture},
    };
    let id = {
        let mut t = setup
            .resources
            .get_mut::<config::Store<Texture>>()
            .expect("");
        t.add(Texture::new(String::from("SOF3.png"), 0, 0, 6, 6))
    };
    setup
        .entity((
            render::Renderable,
            Position::new(1., 2., 3.),
            Shape::builder()
                .unit(shape::Unit::Cube)
                .matrix(Matrix::identity())
                .texture(id)
                .build(),
            traffloat::sun::LightStats::default(),
        ))
        .entity((
            render::Renderable,
            Position::new(1., -2., 3.),
            Shape::builder()
                .unit(shape::Unit::Cube)
                .matrix(Matrix::identity())
                .texture(id)
                .build(),
            traffloat::sun::LightStats::default(),
        ))
        .entity((
            render::Renderable,
            Position::new(-2., 0., 3.),
            Shape::builder()
                .unit(shape::Unit::Cube)
                .matrix(Matrix::identity())
                .texture(id)
                .build(),
            traffloat::sun::LightStats::default(),
        ))
}
