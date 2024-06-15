//! The base structural graph of Traffloat.

use bevy::app;

pub mod building;
pub mod corridor;

/// Maintains graph components.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) { app.add_plugins(corridor::Plugin); }
}
