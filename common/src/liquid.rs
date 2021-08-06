//! Management of liquid in buildings

use derive_new::new;
use legion::world::SubWorld;
use legion::Entity;
use smallvec::SmallVec;
use typed_builder::TypedBuilder;

use crate::clock::{SimulationEvent, SIMULATION_PERIOD};
use crate::def;
use crate::edge;
use crate::time::Instant;
use crate::units::{self, LiquidVolume};
use crate::util;
use crate::SetupEcs;

/// A component attached to entities that house liquid.
#[derive(new, getset::Getters)]
pub struct StorageList {
    /// The list of liquids stored in the entity.
    #[getset(get = "pub")]
    storages: SmallVec<[Entity; 4]>,
}

/// A component attached to storage entities.
#[derive(new, getset::Getters)]
pub struct Storage {
    /// The type of liquid.
    #[getset(get = "pub")]
    liquid: def::liquid::TypeId, // TODO should we optimize this to a runtime integer ID?
}

/// A component attached to storages to inidcate capacity.
#[derive(Debug, Clone, Copy, new, getset::CopyGetters)]
pub struct StorageCapacity {
    /// The maximum liquid size.
    #[getset(get_copy = "pub")]
    total: LiquidVolume,
}

codegen::component_depends! {
    Storage = (
        Storage,
        StorageCapacity,
        StorageSize,
        NextStorageSize,
    ) + ?()
}

/// The size of a liquid storage in the current simulation frame.
#[derive(new, getset::CopyGetters)]
pub struct StorageSize {
    /// The liquid size
    #[getset(get_copy = "pub")]
    size: LiquidVolume,
}

/// The size of a liquid storage in the next simulation frame.
#[derive(new, getset::CopyGetters, getset::MutGetters)]
pub struct NextStorageSize {
    /// The liquid size
    #[getset(get_copy = "pub")]
    #[getset(get_mut = "pub")]
    size: LiquidVolume,
}

/// Interpolates the current graphical size of a storage.
pub fn lerp(current: &StorageSize, next: &NextStorageSize, time: Instant) -> LiquidVolume {
    LiquidVolume(util::lerp(
        current.size.value(),
        next.size.value(),
        (time.since_epoch() % SIMULATION_PERIOD).as_secs() / SIMULATION_PERIOD.as_secs(),
    ))
}

/// A liquid pipe entity.
#[derive(TypedBuilder, getset::CopyGetters)]
pub struct Pipe {
    /// The edge
    #[getset(get_copy = "pub")]
    edge: edge::Id,
    /// The resistance of the pipe.
    #[getset(get_copy = "pub")]
    resistance: f64,
}

/// A component storing the resistance of a pipe.
#[derive(new, getset::CopyGetters)]
pub struct Resistance {
    /// The resistance value,
    /// computed by `length / radius^2`
    #[getset(get_copy = "pub")]
    value: f64,
}

impl Resistance {
    /// Computes the resistance of a pipe.
    pub fn compute(length: f64, radius: f64) -> Self {
        Self::new(length / radius.powi(2))
    }
}

codegen::component_depends! {
    Pipe = (
        Resistance,
    ) + ?()
}

/// A component applied on a node that drives a pipe.
#[derive(TypedBuilder, getset::CopyGetters)]
pub struct LiquidPump {
    /// The force provided by the pump.
    #[getset(get_copy = "pub")]
    force: units::PipeForce,
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
    setup.uses(update_storage_setup)
}
