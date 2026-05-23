use bevy::app::{App, Plugin};
use bevy::ecs::component::Component;
use bevy::math::Vec2;

pub type Vector = Vec2;

#[macro_use]
pub mod util;

pub mod fluid;
pub mod generate;
pub mod graph;
pub mod reactor;
pub mod view;

#[cfg(any(rust_analyzer, doc))]
pub mod docs;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.add_plugins(view::Plug);
        app.add_plugins(graph::Plug);
        app.add_plugins(fluid::Plug);
    }
}

/// Marker component for a root entity in the physics simulation.
///
/// All physics entities that do not have a `linked_spawn` relationship must have this component
/// to facilitate proper teardown when the physics world is unloaded.
#[derive(Component)]
pub struct WorldObject;
