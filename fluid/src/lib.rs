//! Fluids are items that can diffuse between adjacent storages and pipes.
//!
//! "Fluid" is the generalization of gases and liquids.
#![doc = include_str!("../README.md")]

use bevy::app::{self, App};
use bevy::state::state::States;

pub mod config;
pub mod container;
pub mod pipe;
pub mod units;

mod commands;
pub use commands::*;

/// Initializes fluid simulation systems.
pub struct Plugin<St>(pub St);

impl<St: States + Copy> app::Plugin for Plugin<St> {
    fn build(&self, app: &mut App) {
        app.add_plugins((config::Plugin, container::Plugin(self.0), pipe::Plugin(self.0)));
    }
}
