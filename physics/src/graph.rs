use bevy::app::{self, App, Plugin};
use bevy::ecs::schedule::{IntoScheduleConfigs, SystemSet};
use strum::IntoEnumIterator;
use traffloat_util::configure_enum_system_set;

use crate::view;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.add_plugins(building::Plug);
        app.add_plugins(corridor::Plug);
        app.add_plugins(edge::Plug);
        app.add_plugins(facility::Plug);
        app.add_plugins(connection::Plug);
        app.add_plugins(conduit::Plug);

        configure_enum_system_set::<ViewInitSystemSets>(app, app::Update);
        for set in ViewInitSystemSets::iter() {
            app.configure_sets(app::Update, set.in_set(view::InitSystemSets::Graph));
        }
        configure_enum_system_set::<ViewIncrSystemSets>(app, app::Update);
        for set in ViewIncrSystemSets::iter() {
            app.configure_sets(app::Update, set.in_set(view::IncrSystemSets::Graph));
        }
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter)]
pub enum ViewInitSystemSets {
    Building,
    Corridor,
    Facility,
    Conduit,
    Connection,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter)]
pub enum ViewIncrSystemSets {
    Building,
    Corridor,
    Edge,
    Facility,
    Conduit,
    Connection,
}

pub mod building;
pub use building::Building;

pub mod facility;
pub use facility::{Facility, FacilityType, FacilityTypeDef, FacilityTypeInstances};

pub mod corridor;
pub use corridor::Corridor;

pub mod edge;
pub use edge::{BuildingEdges, CorridorEdge, Edge};

pub mod conduit;
pub use conduit::{Conduit, ConduitType};

pub mod connection;
pub use connection::Connection;
