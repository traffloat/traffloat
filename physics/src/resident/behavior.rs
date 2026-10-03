use bevy::app::{App, Plugin};

pub mod pathfind;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) { app.add_plugins(pathfind::Plug); }
}
