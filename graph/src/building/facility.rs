//! An internal structure of a building.

use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use typed_builder::TypedBuilder;

/// Components for a facility.
#[derive(bundle::Bundle, TypedBuilder)]
#[allow(missing_docs)]
pub struct Bundle {
    pub owner: Owner,
}

/// References the owning building for a facility.
#[derive(Component)]
pub struct Owner {
    /// The building in which this facility is installed.
    pub building: Entity,
}
