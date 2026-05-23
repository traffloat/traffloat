use bevy::app::{App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use serde::{Deserialize, Serialize};

use crate::Vector;
use crate::util::AlphaBeta;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) { app.add_plugins(building::Plug); }
}

pub mod building;
pub use building::Building;

pub mod facility;
pub use facility::{
    Facility, FacilityList, FacilityOf, FacilityType, FacilityTypeDef, FacilityTypeInstances,
};

#[derive(Component)]
pub struct Corridor {
    pub endpoints: AlphaBeta<Entity>,

    pub length:       f32,
    pub ambient_area: f32,
}

#[derive(Component)]
pub struct Conduit {
    pub area: f32,
    pub ty:   ConduitType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConduitType {
    FluidPipe,
    PowerCable,
    VehicleRail,
}
