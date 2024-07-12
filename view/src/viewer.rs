//! A viewer entity represents an information subscriber that observes part of the word.

use bevy::ecs::bundle;
use bevy::math::Vec3A;
use bevy::prelude::Component;
use typed_builder::TypedBuilder;

/// Components for a viewer.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    position: Position,
    range:    Range,
}

/// The center of the viewport of a viewer.
#[derive(Component)]
pub struct Position {
    pub position: Vec3A,
}

/// The maximum distance a viewer can observe.
#[derive(Component)]
pub struct Range {
    pub distance: f32,
}
