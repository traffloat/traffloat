//! Management of gas in buildings

use derive_new::new;
use gusket::Gusket;
use legion::world::SubWorld;
use legion::Entity;
use smallvec::SmallVec;
use typed_builder::TypedBuilder;

use crate::clock::{SimulationEvent, SIMULATION_PERIOD};
use crate::def::gas;
use crate::time::Instant;
use crate::units::{self, GasVolume};
use crate::{util, SetupEcs};

/// A component attached to entities that house gas.
#[derive(new, Gusket)]
pub struct StorageList {
    /// The list of gases stored in the entity.
    #[gusket(immut)]
    storages: SmallVec<[(gas::Id, Entity); 4]>,
}

/// A component attached to nodes to inidcate cargo capacity of the node.
#[derive(Debug, Clone, Copy, new, Gusket)]
pub struct StorageCapacity {
    /// The maximum total cargo size.
    #[gusket(immut, copy)]
    total: GasVolume,
}

/// A component attached to storage entities.
#[derive(new, Gusket)]
pub struct Storage {
    /// The type of gas.
    #[gusket(immut, copy)]
    gas: gas::Id,
}

/// The size of a gas storage in the current simulation frame.
#[derive(new, Gusket)]
pub struct StorageSize {
    /// The gas size
    #[gusket(immut, copy)]
    size: GasVolume,
}

/// The size of a gas storage in the next simulation frame.
#[derive(new, Gusket)]
pub struct NextStorageSize {
    /// The gas size
    #[gusket(copy)]
    size: GasVolume,
}

codegen::component_depends! {
    Storage = (
        Storage,
        StorageSize,
        NextStorageSize,
    ) + ?()
}

/// Interpolates the current graphical size of a storage.
pub fn lerp(current: &StorageSize, next: &NextStorageSize, time: Instant) -> GasVolume {
    GasVolume(util::lerp(
        current.size.value(),
        next.size.value(),
        (time.since_epoch() % SIMULATION_PERIOD).as_secs() / SIMULATION_PERIOD.as_secs(),
    ))
}

/// A component applied on a node that drives gas.
#[derive(TypedBuilder, Gusket)]
pub struct Pump {
    /// The force provided by the pump.
    #[gusket(immut, copy)]
    force: units::FanForce,
}

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
