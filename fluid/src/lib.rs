//! Fluids are items that can diffuse between adjacent storages and pipes.
//!
//! "Fluid" is the generalization of gases and liquids.
#![doc = include_str!("../README.md")]

use bevy::app;
use config::Config;

pub mod config;
pub mod container;
pub mod pipe;
pub mod units;

mod commands;
pub use commands::*;

/// Protobuf save structures.
#[allow(missing_docs)]
pub mod proto {
    pub mod save {
        include!(concat!(env!("OUT_DIR"), "/traffloat.save.rs"));
    }
}

pub struct Plugin {
    pub config: Config,
}

impl app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.config.clone());
        app.add_plugins((container::Plugin, pipe::Plugin));
    }
}
