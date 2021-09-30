//! Management of gas in buildings

use derive_new::new;
use legion::world::SubWorld;
use legion::Entity;
use smallvec::SmallVec;
use typed_builder::TypedBuilder;

use crate::clock::{SimulationEvent, SIMULATION_PERIOD};
use crate::time::Instant;
use crate::units::{self, GasVolume};
use crate::{def, util, SetupEcs};

/// A component attached to entities that house gas.
#[derive(new, getset::Getters)]
pub struct StorageList {
    /// The list of gases stored in the entity.
    #[getset(get = "pub")]
    storages: SmallVec<[(def::gas::TypeId, Entity); 4]>,
}

/// A component attached to nodes to inidcate cargo capacity of the node.
#[derive(Debug, Clone, Copy, new, getset::CopyGetters)]
pub struct StorageCapacity {
    /// The maximum total cargo size.
    #[getset(get_copy = "pub")]
    total: GasVolume,
}

/// A component attached to storage entities.
#[derive(new, getset::Getters)]
pub struct Storage {
    /// The type of gas.
    #[getset(get = "pub")]
    gas: def::gas::TypeId, // TODO should we optimize this to a runtime integer ID?
}

/// The size of a gas storage in the current simulation frame.
#[derive(new, getset::CopyGetters)]
pub struct StorageSize {
    /// The gas size
    #[getset(get_copy = "pub")]
    size: GasVolume,
}

/// The size of a gas storage in the next simulation frame.
#[derive(new, getset::CopyGetters, getset::MutGetters)]
pub struct NextStorageSize {
    /// The gas size
    #[getset(get_copy = "pub")]
    #[getset(get_mut = "pub")]
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
#[derive(TypedBuilder, getset::CopyGetters)]
pub struct Pump {
    /// The force provided by the pump.
    #[getset(get_copy = "pub")]
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
