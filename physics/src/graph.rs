use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::schedule::{IntoScheduleConfigs, SystemSet};
use serde::{Deserialize, Serialize};

use crate::Vector;
use crate::util::AlphaBeta;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.add_plugins(building::Plug);
        app.add_plugins(corridor::Plug);
        app.configure_sets(
            app::Update,
            (
                ViewSystemSets::Building.before(ViewSystemSets::Corridor),
                ViewSystemSets::Corridor.before(ViewSystemSets::Facility),
                ViewSystemSets::Facility.before(ViewSystemSets::Pipe),
            ),
        );
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewSystemSets {
    Building,
    Corridor,
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

pub mod conduit;
pub use conduit::{Conduit, ConduitType};
