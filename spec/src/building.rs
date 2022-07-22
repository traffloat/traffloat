//! A building is defines the node type, specifying its functionalities.

use serde::{Deserialize, Serialize};
use xylem::Xylem;

use crate::i18n::I18n;
use crate::reaction::Reaction;
use crate::{cargo, fluid, population, unit};

/// Defines a building type.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub struct Building {
    /// The copy-safe identifier.
    #[xylem(args(new = true))]
    pub id:          Id,
    /// The string identifier.
    #[xylem(serde(default))]
    pub id_str:      IdString,
    /// The display name.
    pub name:        I18n,
    /// A short, one-line description.
    pub summary:     I18n,
    /// A detailed description.
    pub description: I18n,

    /// The storages provided by the building.
    pub storage: Storage,

    /// Number of inhabitants that can be assigned to this building as housing.
    pub housing: unit::PopulationSize,

    pub reactions: Vec<Reaction>,
}

impl_identifiable!(Building);

/// Defines the storages provided by a building.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub struct Storage {
    /// The total cargo capacity.
    pub cargo:      unit::CargoVolume,
    /// The fluid containers.
    pub fluid:      Vec<FluidStorage>,
    /// The roles of inhabitants in this building.
    pub population: Vec<PopulationStorage>,
}

/// A fluid storage.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub struct FluidStorage {
    /// References the fluid storage type.
    pub id: fluid::StorageId,

    /// The volume capacity of this storage.
    /// More fluids can be stored if the fluid is compressible,
    /// creating higher fluid pressure within the container.
    /// The strength of this container is implemented through reactions.
    pub volume: unit::FluidVolume,
}

/// A population storage.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub struct PopulationStorage {
    /// References the population storage type.
    pub id: population::StorageId,

    /// The number of inhabitants allowed in the storage.
    pub capacity: unit::PopulationSize,
}
