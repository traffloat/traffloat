use bevy::app::{App, Plugin};

pub mod plan;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) { app.add_plugins(plan::Plug); }
}
