//! Management of cargo in buildings

use legion::world::SubWorld;
use legion::Entity;
use smallvec::SmallVec;

use crate::clock::{SimulationEvent, SIMULATION_PERIOD};
use crate::config::{Config, Id};
use crate::time::Time;
use crate::units::CargoSize;
use crate::util;
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

/// The size of a cargo storage in the current simulation frame.
#[derive(getset::CopyGetters)]
pub struct StorageSize {
    /// The cargo size
    #[getset(get_copy = "pub")]
    size: CargoSize,
}

/// The size of a cargo storage in the next simulation frame.
#[derive(getset::CopyGetters, getset::MutGetters)]
pub struct NextStorageSize {
    /// The cargo size
    #[getset(get_copy = "pub")]
    #[getset(get_mut = "pub")]
    size: CargoSize,
}

/// Interpolates the current graphical size of a storage.
pub fn lerp(current: &StorageSize, next: NextStorageSize, time: Time) -> CargoSize {
    CargoSize(util::lerp(
        current.size.value(),
        next.size.value(),
        (time % SIMULATION_PERIOD).as_secs() / SIMULATION_PERIOD.as_secs(),
    ))
}

#[codegen::system]
#[write_component(StorageSize)]
#[read_component(NextStorageSize)]
fn update_storage(
    world: &mut SubWorld,
    #[subscriber] sim_sub: impl Iterator<Item = SimulationEvent>,
) {
    use legion::IntoQuery;

    if sim_sub.next().is_none() {
        return;
    }

    for (current, next) in <(&mut StorageSize, &NextStorageSize)>::query().iter_mut(world) {
        current.size = next.size;
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup
}
