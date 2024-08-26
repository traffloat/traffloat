//! Manages the viewers of a Traffloat world.
//!
//! This crate is used for both single-player and multi-player servers.
//! It provides an abstract layer of communication
//! between the simulation modules ("server") and the visualization modules ("client").

use bevy::app::{self, App};

#[macro_use]
mod sid;
pub use sid::Index as SidIndex;

pub mod appearance;

pub mod metrics;
pub mod viewable;
pub mod viewer;

/// Initializes the view framework.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((viewable::Plugin, viewer::Plugin, metrics::Plugin));
    }
}
