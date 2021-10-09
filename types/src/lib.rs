//! Defines common types in traffloat context.

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
    allow(dead_code, unused_imports, unused_variables, clippy::match_single_binding,)
)]
#![cfg_attr(any(doc, not(debug_assertions)), deny(missing_docs))]
#![cfg_attr(
    not(debug_assertions),
    deny(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::dbg_macro,
        clippy::indexing_slicing,
    )
)]

#[macro_use]
mod ratio_def;

pub mod geometry;
pub mod space;
pub mod time;
pub mod units;
