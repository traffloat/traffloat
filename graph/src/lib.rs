//! The base structural graph of Traffloat.
#![doc = include_str!("../README.md")]

use bevy::app::{self, App};

pub mod building;
pub mod corridor;

/// Maintains graph components.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) { app.add_plugins((building::Plugin, corridor::Plugin)); }
}
