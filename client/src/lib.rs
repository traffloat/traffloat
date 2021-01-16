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

use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod app;
mod config;
mod keymap;
mod models;
mod render;
mod time;
use time::Timer;

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

    App::<app::Lifecycle>::new().mount_to_body();
}

fn setup_specs() -> (specs::World, specs::Dispatcher<'static, 'static>) {
    use specs::WorldExt;

    let mut setup = (specs::World::new(), specs::DispatcherBuilder::new());
    setup = common::setup_specs(setup);

    setup = keymap::setup_specs(setup);
    setup = render::setup_specs(setup);

    (setup.0, setup.1.build())
}
