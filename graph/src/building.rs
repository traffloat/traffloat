//! A building in which facilities can be installed.

use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::world::World;
use bevy::transform::components::Transform;
use typed_builder::TypedBuilder;

pub mod facility;

/// Components for a building.
#[derive(bundle::Bundle, TypedBuilder)]
#[allow(missing_docs)]
pub struct Bundle {
    pub position: Transform,
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

pub fn save_buildings(world: &mut World, defs: &mut Vec<prost_types::Any>) {

}
