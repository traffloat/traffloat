//! The base structural graph of Traffloat.
#![doc = include_str!("../README.md")]
use bevy::app;

pub mod building;
pub mod corridor;

/// Protobuf save structures.
#[allow(missing_docs)]
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/traffloat.rs"));

    pub mod save {
        include!(concat!(env!("OUT_DIR"), "/traffloat.save.rs"));
    }
}

/// Maintains graph components.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) { app.add_plugins(corridor::Plugin); }
}
