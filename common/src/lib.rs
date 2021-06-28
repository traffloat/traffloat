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

#[macro_use]
mod macros;

pub mod config;
pub use config::Config;
pub mod graph;
pub mod proto;
pub mod shape;
pub mod space;
pub mod sun;
pub mod time;
pub mod units;
mod util;
pub use util::*;

pub use codegen::{Legion, SetupEcs};

/// Initializes common modules.
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup
        .resource(codegen::Perf::default())
        .uses(config::setup_ecs)
        .uses(time::setup_ecs)
        .uses(shape::setup_ecs)
        .uses(graph::setup_ecs)
        .uses(sun::setup_ecs)
}
