//! An internal structure of a corridor.

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
    /// The corridor in which this duct is installed.
    pub corridor: Entity,
}
