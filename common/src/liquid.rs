//! Management of liquid in buildings

use derive_new::new;
use legion::world::SubWorld;
use legion::Entity;
use smallvec::SmallVec;
use typed_builder::TypedBuilder;

use crate::clock::{SimulationEvent, SIMULATION_PERIOD};
use crate::config;
use crate::def::{self, liquid::TypeId};
use crate::node;
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
#[derive(new, getset::Getters, getset::Setters)]
pub struct Storage {
    /// The type of liquid.
    #[getset(get = "pub")]
    #[getset(set = "pub")]
    liquid: TypeId,
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
        node::Child,
        StorageCapacity,
        StorageSize,
        NextStorageType,
        NextStorageSize,
    ) + ?()
}

/// The size of a liquid storage in the current simulation frame.
#[derive(new, getset::CopyGetters)]
pub struct StorageSize {
    /// The liquid size.
    #[getset(get_copy = "pub")]
    size: LiquidVolume,
}

/// The type of a liquid storage in the next simulation frame.
#[derive(new, getset::Getters, getset::Setters)]
pub struct NextStorageType {
    /// The liquid type.
    #[getset(get = "pub")]
    #[getset(set = "pub")]
    ty: TypeId,
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
///
/// Note that [`src_entity`] and [`dest_entity`] are entities of the liquid storage, not the node
/// itself.
#[derive(new, getset::CopyGetters)]
pub struct Pipe {
    /// Entity of the source storage
    #[getset(get_copy = "pub")]
    src_entity: Entity,
    /// Entity of the destination storage
    #[getset(get_copy = "pub")]
    dest_entity: Entity,
}

/// A component storing the resistance of a pipe.
#[derive(new, getset::CopyGetters)]
pub struct PipeResistance {
    /// The resistance value,
    /// computed by `length / radius^2`
    #[getset(get_copy = "pub")]
    value: f64,
}

impl PipeResistance {
    /// Computes the resistance of a pipe.
    pub fn compute(length: f64, radius: f64) -> Self {
        Self::new(length / radius.powi(2))
    }
}

/// A component storing the current flow of a pipe.
#[derive(Default, getset::Getters, getset::CopyGetters, getset::Setters)]
pub struct PipeFlow {
    /// The type of liquid flowing over the pipe in the current simulation frame.
    #[getset(get = "pub")]
    #[getset(set = "pub")]
    ty: Option<TypeId>,
    /// The flow rate over the pipe in the current simulation frame.
    #[getset(get_copy = "pub")]
    #[getset(set = "pub")]
    value: LiquidVolume,
}

codegen::component_depends! {
    Pipe = (
        Pipe,
        PipeResistance,
        PipeFlow,
    ) + ?()
}

/// A component applied on a node that drives a pipe.
#[derive(Debug, TypedBuilder, getset::CopyGetters)]
pub struct Pump {
    /// The force provided by the pump.
    #[getset(get_copy = "pub")]
    force: units::PipeForce,
}

#[codegen::system]
#[read_component(Pipe)]
#[read_component(PipeResistance)]
#[read_component(Pump)]
#[read_component(Storage)]
#[read_component(StorageCapacity)]
#[read_component(StorageSize)]
#[read_component(node::Child)]
#[write_component(NextStorageSize)]
#[write_component(NextStorageType)]
#[write_component(PipeFlow)]
fn simulate_pipes(
    world: &mut SubWorld,
    #[resource] config: &config::Scalar,
    #[resource(no_init)] def: &def::GameDefinition,
    #[subscriber] sim_sub: impl Iterator<Item = SimulationEvent>,
) {
    use legion::{world::ComponentError, EntityStore, IntoQuery};

    if sim_sub.next().is_none() {
        return;
    }

    let mut query = <(&Pipe, &PipeResistance, &mut PipeFlow)>::query();
    let (mut query_world, mut entry_world) = world.split_for_query(&query);
    for (pipe, resistance, flow) in query.iter_mut(&mut query_world) {
        struct FetchEndpoint {
            ty: TypeId,
            force: units::PipeForce,
            volume: units::LiquidVolume,
            empty: units::LiquidVolume,
            viscosity: units::LiquidViscosity,
        }

        let fetch_endpoint = |storage_entity: Entity| {
            let entry = entry_world
                .entry_ref(storage_entity)
                .expect("Pipe references nonexistent endpoint");

            let storage = entry
                .get_component::<Storage>()
                .expect("Pipe endpoint does not have Storage component");
            let ty = storage.liquid();

            let parent = entry
                .get_component::<node::Child>()
                .expect("Pipe endpoint does not have node::Child component");
            let parent_entry = entry_world
                .entry_ref(parent.parent())
                .expect("Storage references nonexistent parent");
            let pump = parent_entry.get_component::<Pump>();
            let force = match pump {
                Ok(pump) => pump.force(),
                Err(ComponentError::NotFound { .. }) => units::PipeForce(0.),
                Err(ComponentError::Denied { .. }) => unreachable!(),
            };
            // TODO multiply force by power

            let size = entry
                .get_component::<StorageSize>()
                .expect("Pipe endpoint does not have StorageSize component");
            let capacity = entry
                .get_component::<StorageCapacity>()
                .expect("Pipe endpoint does not have StorageCapacity component");

            let viscosity =
                def.liquid().get(ty).expect("Storage references undefined liquid").viscosity();

            FetchEndpoint {
                ty: ty.clone(),
                force,
                volume: size.size(),
                empty: capacity.total() - size.size(),
                viscosity,
            }
        };

        let src = fetch_endpoint(pipe.src_entity());
        let dest = fetch_endpoint(pipe.dest_entity());

        let sum_ty = if src.volume < config.negligible_volume {
            dest.ty.clone()
        } else {
            def.liquid_mixer().mix(&src.ty, &dest.ty).clone()
        };

        let force = src.force + dest.force;

        let mut rate = force.value() / resistance.value() / src.viscosity.value();
        rate = rate.min(src.volume.value()).min(dest.empty.value());
        let rate = units::LiquidVolume(rate);

        flow.set_ty(Some(src.ty.clone()));
        flow.set_value(rate);

        {
            let mut src_entity = entry_world
                .entry_mut(pipe.src_entity())
                .expect("Pipe references nonexistent endpoint");
            let next = src_entity
                .get_component_mut::<NextStorageSize>()
                .expect("Pipe endpoint does not have NextStorageSize component");
            *next.size_mut() -= rate;
        }
        {
            let mut dest_entity = entry_world
                .entry_mut(pipe.dest_entity())
                .expect("Pipe references nonexistent endpoint");
            {
                let next = dest_entity
                    .get_component_mut::<NextStorageType>()
                    .expect("Pipe endpoint does not have NextStorageType component");
                next.set_ty(sum_ty);
            }
            {
                let next = dest_entity
                    .get_component_mut::<NextStorageSize>()
                    .expect("Pipe endpoint does not have NextStorageSize component");
                *next.size_mut() += rate;
            }
        }
    }
}

#[codegen::system]
#[write_component(Storage)]
#[write_component(StorageSize)]
#[read_component(NextStorageType)]
#[write_component(NextStorageSize)]
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
    for (current, next) in <(&mut Storage, &NextStorageType)>::query().iter_mut(world) {
        current.set_liquid(next.ty.clone());
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(simulate_pipes_setup).uses(update_storage_setup)
}
