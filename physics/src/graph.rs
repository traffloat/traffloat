use bevy::app::{App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;

use crate::Vector;
use crate::util::AlphaBeta;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
pub struct Building {
    pub position:       Vector,
    pub radius:         f32,
    pub ambient_volume: f32,
}

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
pub struct Corridor {
    pub endpoints: AlphaBeta<Entity>,

    pub length:       f32,
    pub ambient_area: f32,
}

#[derive(Component)]
pub struct Conduit {
    pub area: f32,
}
