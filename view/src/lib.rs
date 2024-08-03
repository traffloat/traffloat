//! Manages the viewers of a Traffloat world.
//!
//! This crate is used for both single-player and multi-player servers.

use bevy::app::{self, App};

pub mod building;
pub mod metrics;
pub mod viewable;
pub mod viewer;

/// Initializes the view framework.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((viewable::Plugin, building::Plugin, metrics::Plugin));
    }
}
