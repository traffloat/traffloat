use bevy::app::{App, Plugin};
use bevy::math::Vec2;

pub type Vector = Vec2;

#[macro_use]
pub mod util;

pub mod fluid;
pub mod graph;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.add_plugins(graph::Plug);
        app.add_plugins(fluid::Plug);
    }
}
