use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::system::EntityCommand;
use bevy::ecs::world::EntityWorldMut;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::vehicle::def::GaugeSize;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) { app.register_type::<Rail>(); }
}

#[derive(Debug, Clone, Component, Serialize, Deserialize, Reflect)]
pub struct Rail {
    pub gauge_size:    GaugeSize,
    pub electrified:   bool,
    pub maximum_speed: f32,
    // TODO further restriction labels
}

pub struct AddRailCommand {
    pub rail: Rail,
}

impl EntityCommand for AddRailCommand {
    type Out = ();

    fn apply(self, mut entity: EntityWorldMut) { entity.insert(self.rail); }
}
