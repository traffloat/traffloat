//! Fluids are items that can diffuse between adjacent storages and pipes.
//!
//! "Fluid" is the generalization of gases and liquids.
#![doc = include_str!("../README.md")]

use bevy::app;
use config::TypeDefs;

pub mod config;
pub mod container;
pub mod pipe;
pub mod units;

pub struct Plugin {
    pub defs: TypeDefs,
}

impl app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.defs.clone());
        app.add_plugins(container::Plugin);
    }
}
