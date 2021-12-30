//! Management of liquid in buildings

use std::collections::{btree_map, BTreeMap};

use derive_new::new;
use gusket::Gusket;
use legion::world::SubWorld;
use legion::Entity;
use smallvec::SmallVec;
use typed_builder::TypedBuilder;

use crate::clock::{SimulationEvent, SIMULATION_PERIOD};
use crate::def::liquid;
use crate::time::Instant;
use crate::units::{self, LiquidVolume};
use crate::{config, def, node, save, util, SetupEcs};

/// A data structure storing liquid mixing recipes.
#[derive(Default)]
pub struct RecipeMap {
    map:     BTreeMap<RecipeKey, liquid::Id>,
    default: Option<liquid::Id>,
}

impl RecipeMap {
    /// Push a formula definition.
    pub fn define(&mut self, def: &liquid::Formula) -> anyhow::Result<()> {
        let key = RecipeKey::new(def.augend(), def.addend());
        match self.map.entry(key) {
            btree_map::Entry::Vacant(entry) => {
                entry.insert(def.sum());
                Ok(())
            }
            btree_map::Entry::Occupied(_) => anyhow::bail!("Duplicate recipe key {:?}", key),
        }
    }

    /// Set the default formula definition.
    pub fn define_default(&mut self, def: &liquid::DefaultFormula) -> anyhow::Result<()> {
        anyhow::ensure!(self.default.is_none(), "Duplicate default recipe");
        self.default = Some(def.sum());
        Ok(())
    }

    /// Evaluate the mixing result of two liquids.
    pub fn mix(&self, augend: liquid::Id, addend: liquid::Id) -> liquid::Id {
        match self.map.get(&RecipeKey::new(augend, addend)) {
            Some(&sum) => sum,
            None => self.default(),
        }
    }

    /// The default liquid mixing output type.
    pub fn default(&self) -> liquid::Id { self.default.expect("Uninitialized recipe default") }
}

/// A recipe key.
///
/// `less` is always less than `greater`
/// to ensure the commutative property of the key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RecipeKey {
    less:    liquid::Id,
    greater: liquid::Id,
}

impl RecipeKey {
    /// Create a new, sorted `RecipeKey`
    fn new(a: liquid::Id, b: liquid::Id) -> Self {
        if a < b {
            Self { less: a, greater: b }
        } else {
            Self { less: b, greater: a }
        }
    }
}

/// A component attached to entities that house liquid.
#[derive(new, Gusket)]
pub struct StorageList {
    /// The list of liquids stored in the entity.
    #[gusket(immut)]
    storages: SmallVec<[Entity; 4]>,
}

/// A component attached to storage entities.
#[derive(new, Gusket)]
pub struct Storage {
    /// The type of liquid.
    #[gusket(copy)]
    liquid: liquid::Id,
}

/// A component attached to storages to inidcate capacity.
#[derive(Debug, Clone, Copy, new, Gusket)]
pub struct StorageCapacity {
    /// The maximum liquid size.
    #[gusket(copy)]
    total: LiquidVolume,
}

codegen::component_depends! {
    Storage = (
        Storage,
        StorageCapacity,
        StorageSize,
        NextStorageType,
        NextStorageSize,
        def::lang::Item,
        node::Child,
    ) + ?()
}

/// The size of a liquid storage in the current simulation frame.
#[derive(new, Gusket)]
pub struct StorageSize {
    /// The liquid size.
    #[gusket(copy)]
    size: LiquidVolume,
}

/// The type of a liquid storage in the next simulation frame.
#[derive(new, Gusket)]
pub struct NextStorageType {
    /// The liquid type.
    #[gusket(copy)]
    ty: liquid::Id,
}

/// The size of a liquid storage in the next simulation frame.
#[derive(new, Gusket)]
pub struct NextStorageSize {
    /// The liquid size
    #[gusket(copy)]
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
#[derive(new, Gusket)]
pub struct Pipe {
    /// Entity of the source storage
    #[gusket(copy)]
    src_entity:  Entity,
    /// Entity of the destination storage
    #[gusket(copy)]
    dest_entity: Entity,
}

/// A component storing the resistance of a pipe.
#[derive(new, Gusket)]
pub struct PipeResistance {
    /// The resistance value,
    /// computed by `length / radius^2`
    #[gusket(copy)]
    value: f64,
}

impl PipeResistance {
    /// Computes the resistance of a pipe.
    pub fn compute(length: f64, radius: f64) -> Self { Self::new(length / radius.powi(2)) }
}

/// A component storing the current flow of a pipe.
#[derive(Default, Gusket)]
pub struct PipeFlow {
    /// The type of liquid flowing over the pipe in the current simulation frame.
    #[gusket(copy)]
    ty:    Option<liquid::Id>,
    /// The flow rate over the pipe in the current simulation frame.
    #[gusket(copy)]
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
#[derive(Debug, TypedBuilder, Gusket)]
pub struct Pump {
    /// The force provided by the pump.
    #[gusket(copy)]
    force: units::PipeForce,
}

#[codegen::system(Simulate)]
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
    #[resource(no_init)] def: &save::GameDefinition,
    #[subscriber] sim_sub: impl Iterator<Item = SimulationEvent>,
) {
    use legion::world::ComponentError;
    use legion::{EntityStore, IntoQuery};

    if sim_sub.next().is_none() {
        return;
    }

    let mut query = <(&Pipe, &PipeResistance, &mut PipeFlow)>::query();
    let (mut query_world, mut entry_world) = world.split_for_query(&query);
    for (pipe, resistance, flow) in query.iter_mut(&mut query_world) {
        struct FetchEndpoint {
            ty:        liquid::Id,
            force:     units::PipeForce,
            volume:    units::LiquidVolume,
            empty:     units::LiquidVolume,
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

            let viscosity = def[ty].viscosity();

            FetchEndpoint {
                ty,
                force,
                volume: size.size(),
                empty: capacity.total() - size.size(),
                viscosity,
            }
        };

        let src = fetch_endpoint(pipe.src_entity());
        let dest = fetch_endpoint(pipe.dest_entity());

        let sum_ty = if src.volume < config.negligible_volume {
            dest.ty
        } else {
            def.liquid_recipes().mix(src.ty, dest.ty)
        };

        let force = src.force + dest.force;

        let mut rate = force.value() / resistance.value() / src.viscosity.value();
        rate = rate.min(src.volume.value()).min(dest.empty.value());
        let rate = units::LiquidVolume(rate);

        flow.set_ty(Some(src.ty));
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

#[codegen::system(PreSimulate)]
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
        current.set_liquid(next.ty);
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(simulate_pipes_setup).uses(update_storage_setup)
}
