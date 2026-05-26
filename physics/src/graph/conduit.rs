use bevy::app::{App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use serde::{Deserialize, Serialize};

use crate::Vector;
use crate::util::AlphaBeta;

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
