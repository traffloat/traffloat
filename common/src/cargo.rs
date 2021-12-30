//! Management of cargo in buildings

use derive_new::new;
use gusket::Gusket;
use legion::world::SubWorld;
use legion::Entity;
use smallvec::SmallVec;

use crate::clock::{SimulationEvent, SIMULATION_PERIOD};
use crate::def::cargo;
use crate::time::Instant;
use crate::units::CargoSize;
use crate::{util, SetupEcs};

/// A component attached to nodes to indicate cargo in the node.
#[derive(new, Gusket)]
pub struct StorageList {
    /// The list of cargos stored in the entity.
    #[gusket(immut)]
    storages: SmallVec<[(cargo::Id, Entity); 4]>,
}

/// A component attached to nodes to inidcate cargo capacity of the node.
#[derive(Debug, Clone, Copy, new, Gusket)]
pub struct StorageCapacity {
    /// The maximum total cargo size.
    #[gusket(immut, copy)]
    total: CargoSize,
}

/// A component attached to storage entities.
#[derive(new, Gusket)]
pub struct Storage {
    /// The type of cargo.
    #[gusket(immut, copy)]
    cargo: cargo::Id,
}

/// The size of a cargo storage in the current simulation frame.
#[derive(new, Gusket)]
pub struct StorageSize {
    /// The cargo size
    #[gusket(immut, copy)]
    size: CargoSize,
}

/// The size of a cargo storage in the next simulation frame.
#[derive(new, Gusket)]
pub struct NextStorageSize {
    /// The cargo size
    #[gusket(immut, copy)]
    size: CargoSize,
}

codegen::component_depends! {
    Storage = (
        Storage,
        StorageSize,
        NextStorageSize,
    ) + ?()
}

/// Interpolates the current graphical size of a storage.
pub fn lerp(current: &StorageSize, next: &NextStorageSize, time: Instant) -> CargoSize {
    CargoSize(util::lerp(
        current.size.value(),
        next.size.value(),
        (time.since_epoch() % SIMULATION_PERIOD).as_secs() / SIMULATION_PERIOD.as_secs(),
    ))
}

/// Copy next.size into current.size
#[codegen::system(PreSimulate)]
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
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs { setup.uses(update_storage_setup) }
