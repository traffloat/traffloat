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
#![cfg_attr(
    not(debug_assertions),
    deny(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::dbg_macro,
        clippy::indexing_slicing,
    )
)]
#![feature(bool_to_option, vecdeque_binary_search)]

use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod app;
mod camera;
mod config;
mod input;
mod render;
mod util;

#[wasm_bindgen(start)]
pub fn run_app() {
    console_error_panic_hook::set_once();

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

fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup
        .uses(traffloat::setup_ecs)
        .uses(input::setup_ecs)
        .uses(camera::setup_ecs)
}
