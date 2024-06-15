//! A building in which facilities can be installed.

use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::math::Vec3A;
use typed_builder::TypedBuilder;

pub mod facility;

/// Components for a building.
#[derive(bundle::Bundle, TypedBuilder)]
#[allow(missing_docs)]
pub struct Bundle {
    pub position: Position,
}

/// Reference position of a building.
#[derive(Component)]
pub struct Position {
    /// Position relative to the global origin.
    pub vec: Vec3A,
}

/// List of facilities in a building.
#[derive(Component)]
pub struct FacilityList {
    /// Non-ambient facilities in this building.
    /// The order of entities in this list has no significance.
    pub facilities: Vec<Entity>, // entities with facility components

    /// The ambient space for this building.
    pub ambient: Entity,
}
