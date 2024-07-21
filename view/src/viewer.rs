//! A viewer entity represents an information subscriber that observes part of the word.

use bevy::ecs::bundle;
use bevy::prelude::Component;
use bevy::transform::components::Transform;
use typed_builder::TypedBuilder;

/// Components for a viewer.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    position: Transform,
    range:    Range,
}

/// The maximum distance a viewer can observe.
///
/// Due to optimization concerns, the distance is interpreted as max-norm instead of 2-norm.
#[derive(Component)]
pub struct Range {
    pub distance: f32,
}
