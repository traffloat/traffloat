use bevy::app::{self, App, Plugin};
use bevy::ecs::schedule::SystemSet;

use crate::util;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.add_plugins(building::Plug);
        app.add_plugins(corridor::Plug);
        app.add_plugins(edge::Plug);
        app.add_plugins(facility::Plug);
        app.add_plugins(connection::Plug);
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
    Connection,
}

pub mod building;
pub use building::Building;

pub mod facility;
pub use facility::{Facility, FacilityType, FacilityTypeDef, FacilityTypeInstances};

pub mod corridor;
pub use corridor::Corridor;

pub mod edge;

pub mod conduit;
pub use conduit::{Conduit, ConduitType};

pub mod connection;
pub use connection::Connection;
