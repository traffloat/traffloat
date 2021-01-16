//! Common library for server and client

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
        missing_docs,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::dbg_macro,
        clippy::indexing_slicing,
    )
)]

pub mod liquid;
pub mod proto;
pub mod reaction;
pub mod shape;
pub mod terminal;
pub mod time;
pub mod types;
mod util;
pub use util::*;

pub type Setup = (specs::World, specs::DispatcherBuilder<'static, 'static>);
pub fn setup_specs(mut setup: Setup) -> Setup {
    setup = liquid::setup_specs(setup);
    setup = reaction::setup_specs(setup);
    setup = shape::setup_specs(setup);
    setup = terminal::setup_specs(setup);
    setup = time::setup_specs(setup);
    setup
}
