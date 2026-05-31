use bevy::app::{App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::Vector;
use crate::util::AlphaBeta;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        /* TODO
        app.add_systems(app::Update, init_viewer_system.in_set(view::SendUpdatesSystemSet::Init));
        app.add_systems(
            app::Update,
            (basic_incr_viewer_system, full_incr_viewer_system)
                .chain()
                .in_set(super::ViewSystemSets::Facility)
                .in_set(view::SendUpdatesSystemSet::Incr),
        );
        */
    }
}

#[derive(Component, Reflect)]
pub struct Conduit {
    pub area: f32,
    pub ty:   ConduitType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum ConduitType {
    FluidPipe,
    PowerCable,
    VehicleRail,
}
