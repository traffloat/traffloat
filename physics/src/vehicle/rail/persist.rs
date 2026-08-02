use bevy::ecs::query::QueryData;
use serde::{Deserialize, Serialize};

use crate::graph::conduit;
use crate::vehicle::{Rail, rail};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEntry {
    pub rail:         Rail,
    pub reserved_dir: Option<rail::ReservedDirection>,
}

impl StorageEntry {
    pub fn from_component(comp: OutputQueryDataItem) -> Self {
        Self {
            rail:         comp.rail.clone(),
            reserved_dir: comp.reservation.inner.map(|inner| inner.direction),
        }
    }

    pub fn into_conduit_typed_spawn(self) -> conduit::TypedSpawn {
        conduit::TypedSpawn::VehicleRail {
            rail:         self.rail,
            reserved_dir: self.reserved_dir,
        }
    }
}

#[derive(QueryData)]
pub struct OutputQueryData {
    rail:        &'static Rail,
    reservation: &'static rail::Reservation,
}
