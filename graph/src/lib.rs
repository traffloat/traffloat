//! The base structural graph of Traffloat.
#![doc = include_str!("../README.md")]
use bevy::app;

pub mod building;
pub mod corridor;

/// Protobuf structures.
pub mod proto {
    /// Save files.
    pub mod save {
        include!(concat!(env!("OUT_DIR"), "/traffloat.save.rs"));
    }
}

/// Maintains graph components.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) { app.add_plugins(corridor::Plugin); }
}
