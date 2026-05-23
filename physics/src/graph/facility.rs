use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;

#[derive(Component)]
pub struct Facility {
    pub volume: f32,
}

#[derive(Component)]
#[relationship_target(relationship = FacilityOf, linked_spawn)]
pub struct FacilityList(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = FacilityList)]
pub struct FacilityOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = FacilityType)]
pub struct FacilityTypeInstances(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = FacilityTypeInstances)]
pub struct FacilityType(pub Entity);

#[derive(Component)]
pub struct FacilityTypeDef {
    pub display: String,
    pub volume:  f32,
}
