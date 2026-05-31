use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::schedule::{IntoScheduleConfigs, SystemSet};
use serde::{Deserialize, Serialize};

use crate::util::AlphaBeta;
use crate::{Vector, util};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.add_plugins(building::Plug);
        app.add_plugins(corridor::Plug);
        app.add_plugins(edge::Plug);
        app.add_plugins(facility::Plug);
        app.add_plugins(conduit::Plug);
        util::configure_enum_system_set::<ViewSystemSets>(app, app::Update);
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter)]
pub enum ViewSystemSets {
    Building,
    Corridor,
    Edge,
    Facility,
    Pipe,
}

pub mod building;
pub use building::Building;

pub mod facility;
pub use facility::{
    Facility, FacilityList, FacilityOf, FacilityType, FacilityTypeDef, FacilityTypeInstances,
};

pub mod corridor;
pub use corridor::Corridor;

pub mod edge;

pub mod conduit;
pub use conduit::{Conduit, ConduitType};
