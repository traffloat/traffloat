//! When a building is in range of a viewer,
//! the viewer receives events related to that building.

use bevy::app::{self, App};
use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::math::Mat4;
use bevy::prelude::Entity;
use typed_builder::TypedBuilder;

use crate::viewable;

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, _: &mut App) {}
}

/// Viewee-related extension components for a building.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    viewable:     viewable::Bundle,
    render_model: RenderModel,
}

/// The model that clients should render the building with.
#[derive(Component)]
pub struct RenderModel {
    /// Reference to the building model entity.
    pub entity:    Entity,
    /// Transformation matrix from model coordinates to building coordinates
    /// relative to the building position.
    pub transform: Mat4,
}
