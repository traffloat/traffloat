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

pub struct Plugin {
    pub config: Config,
}

impl app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.config.clone());
        app.add_plugins((container::Plugin, pipe::Plugin));
    }
}
