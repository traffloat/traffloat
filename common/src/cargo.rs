//! Management of cargo in buildings

use legion::Entity;
use smallvec::SmallVec;

use crate::config::{Config, Id};
use crate::time::{Time, SIMULATION_PERIOD};
use crate::units::CargoSize;
use crate::util::lerp;
use crate::SetupEcs;

/// A configuration of cargo.
///
/// Attributes associated with the cargo is located in other components.
#[derive(Debug)]
pub struct Cargo;

impl Config for Cargo {}

/// A component attached to entities that house cargo.
#[derive(getset::Getters)]
pub struct StorageList {
    /// The list of cargos stored in the entity.
    #[getset(get = "pub")]
    storages: SmallVec<[(Id<Cargo>, Entity); 4]>,
}

/// A component attached to storage entities.
#[derive(getset::CopyGetters)]
pub struct Storage {
    /// The type of cargo
    #[getset(get_copy = "pub")]
    cargo: Id<Cargo>,
    /// The maximum amount of the cargo in the storage
    #[getset(get_copy = "pub")]
    capacity: CargoSize,
}

/// Stores the current state of a storage.
#[derive(getset::CopyGetters, getset::Setters)]
pub struct StorageState {
    /// The amount of cargo in the storage.
    #[getset(get_copy = "pub")]
    size: CargoSize,
    /// The amount of cargo in the storage in the next simulation frame.
    #[getset(get_copy = "pub")]
    #[getset(set = "pub")]
    next_size: CargoSize,
}

impl StorageState {
    /// Push the next size as the current size.
    ///
    /// This starts a new simulation frame.
    pub fn push(&mut self) {
        self.size = self.next_size;
    }

    /// Interpolate the storage size at the given time.
    pub fn now(&self, time: Time) -> CargoSize {
        CargoSize(lerp(
            self.size.0,
            self.next_size.0,
            (time % SIMULATION_PERIOD).as_secs() / SIMULATION_PERIOD.as_secs(),
        ))
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup
}
