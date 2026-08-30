//! Reactor refers to a [facility](crate::graph::facility)
//! that processes [reactions](crate::reaction).

use std::time::Duration;

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::{IntoScheduleConfigs, SystemSet};
use bevy::ecs::system::{Local, Query, Res, SystemParam};
use bevy::ecs::world::World;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};
use traffloat_util::{QueryExt, duration_to_timesteps};

use crate::persist::AppExt;
use crate::{CleanupAppExt, fluid, reaction, resident};

mod persist;
pub use persist::Persist;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Facility>();

        app.register_persistable(Persist);

        app.init_resource::<Types>();
        app.init_resource::<Conf>();
        app.add_systems(
            app::FixedUpdate,
            execute_system.in_set(ExecuteSystemSet).in_set(fluid::ModifySystemSets::Reactor),
        );
        app.add_cleanup_hook(Types::cleanup_hook);
    }
}

#[derive(Resource)]
pub struct Conf {
    pub execution_timestep: u32,
}

impl Default for Conf {
    fn default() -> Self {
        Self { execution_timestep: const { duration_to_timesteps(Duration::from_millis(250)) } }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct ExecuteSystemSet;

#[derive(SystemParam)]
struct ExecuteSystemParams<'w, 's> {
    fluid_storage: Query<'w, 's, &'static mut fluid::Storage>,
    resident:
        Query<'w, 's, (&'static mut resident::Attributes, &'static resident::InteractingWith)>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ExecuteFacilityData {
    entity:            Entity,
    facility:          &'static Facility,
    facility_status:   &'static mut FacilityStatus,
    interaction_slots: Option<&'static resident::InteractingResidents>,
}

// In the future we may optimize this module to use dynamic components and systems
// for better performance and parallelism,
// but for now we will keep it simple and use a single component for reactors
// and iterate over all reactors in series.
fn execute_system(
    conf: Res<Conf>,
    mut next_step: Local<u32>,
    reactor_query: Query<ExecuteFacilityData>,
    types: Res<Types>,
    mut params: ExecuteSystemParams,
) {
    *next_step += 1;
    *next_step %= conf.execution_timestep;
    if *next_step != 0 {
        return;
    }

    for mut reactor in reactor_query {
        let def = types.get(reactor.facility.id);
        execute_rule(&mut reactor, def, &mut params);
    }
}

fn execute_rule(
    reactor: &mut ExecuteFacilityDataItem,
    def: &TypeDef,
    params: &mut ExecuteSystemParams,
) {
    let efficiency = reaction::execute_once(
        params,
        reactor,
        &def.inputs,
        &def.catalysts,
        &def.outputs,
        reactor.facility.efficiency_cap,
        1.0,
    );
    reactor.facility_status.efficiency = efficiency;
}

/// Component on facilities.
#[derive(Debug, Component, Reflect)]
#[require(FacilityStatus)]
pub struct Facility {
    pub id:             TypeId,
    /// The maximum efficiency as configured by the player.
    pub efficiency_cap: f32,
    pub ports:          Ports,
}

/// Component on facilities.
#[derive(Debug, Default, Component, Reflect)]
pub struct FacilityStatus {
    /// The facility efficiency in the last timestep.
    pub efficiency: f32,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Reflect,
)]
pub struct TypeId(pub u32);

#[derive(Resource, Default, Reflect)]
pub struct Types {
    pub types: Vec<TypeDef>,
}

impl Types {
    #[must_use]
    pub fn get(&self, id: TypeId) -> &TypeDef {
        self.types.get(id.0 as usize).expect("got invalid reactor type reference")
    }

    pub fn push(&mut self, def: TypeDef) -> TypeId {
        let id = u32::try_from(self.types.len()).expect("too many reactor types");
        self.types.push(def);
        TypeId(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (TypeId, &TypeDef)> {
        self.types
            .iter()
            .enumerate()
            .map(|(i, def)| (TypeId(u32::try_from(i).expect("too many reactor types")), def))
    }

    fn cleanup_hook(world: &mut World) { world.resource_mut::<Types>().types.clear(); }
}

/// A component on facilities.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct TypeDef {
    pub inputs:    Vec<Input>,
    pub outputs:   Vec<Output>,
    pub catalysts: Vec<Catalyst>,
}

#[derive(Debug, Reflect)]
pub struct Ports {
    pub fluid_storages: Vec<Option<Entity>>,
}

/// A reference to an entry in [`Ports::fluid_storages`].
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct FluidPortSelector {
    pub port: u32,
}

impl<'pw, 'ps, 'dw, 'ds>
    reaction::FluidStorageSelector<ExecuteSystemParams<'pw, 'ps>, ExecuteFacilityDataItem<'dw, 'ds>>
    for FluidPortSelector
{
    fn select<R>(
        &self,
        params: &ExecuteSystemParams<'pw, 'ps>,
        data: &ExecuteFacilityDataItem<'dw, 'ds>,
        then: impl FnOnce(&fluid::Storage) -> R,
    ) -> Option<R> {
        match data
            .facility
            .ports
            .fluid_storages
            .get(usize::try_from(self.port).expect("usize >= u32"))
        {
            Some(&Some(entity)) => params.fluid_storage.log_get(entity).map(then),
            Some(None) => None,
            None => {
                tracing::warn!("Reference to undefined port {}", self.port);
                None
            }
        }
    }

    fn select_mut<R>(
        &self,
        params: &mut ExecuteSystemParams<'pw, 'ps>,
        data: &mut ExecuteFacilityDataItem<'dw, 'ds>,
        then: impl FnOnce(&mut fluid::Storage) -> R,
    ) -> Option<R> {
        match data
            .facility
            .ports
            .fluid_storages
            .get(usize::try_from(self.port).expect("usize >= u32"))
        {
            Some(&Some(entity)) => {
                params.fluid_storage.log_get_mut(entity).map(|mut storage| then(&mut storage))
            }
            Some(None) => None,
            None => {
                tracing::warn!("Reference to undefined port {}", self.port);
                None
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ResidentSlotSelector {
    pub slot_index: usize,
}

impl<'pw, 'ps, 'dw, 'ds>
    reaction::ResidentSelector<ExecuteSystemParams<'pw, 'ps>, ExecuteFacilityDataItem<'dw, 'ds>>
    for ResidentSlotSelector
{
    fn for_each_attributes(
        &self,
        params: &ExecuteSystemParams<'pw, 'ps>,
        data: &ExecuteFacilityDataItem<'dw, 'ds>,
        mut then: impl FnMut(&resident::Attributes, Entity),
    ) {
        data.interaction_slots.iter().flat_map(|slots| slots.iter()).for_each(|entity| {
            let Some((attrs, with)) = params.resident.log_get(entity) else { return };
            debug_assert_eq!(with.facility, data.entity);
            if with.slot_index == self.slot_index {
                then(attrs, entity);
            }
        });
    }

    fn for_each_attributes_mut(
        &self,
        params: &mut ExecuteSystemParams<'pw, 'ps>,
        data: &mut ExecuteFacilityDataItem<'dw, 'ds>,
        mut then: impl FnMut(&mut resident::Attributes, Entity),
    ) {
        data.interaction_slots.iter().flat_map(|slots| slots.iter()).for_each(|entity| {
            let Some((mut attrs, with)) = params.resident.log_get_mut(entity) else { return };
            debug_assert_eq!(with.facility, data.entity);
            if with.slot_index == self.slot_index {
                then(&mut attrs, entity);
            }
        });
    }
}

reaction::define_ruleset! {
    [P = ExecuteSystemParams, D = ExecuteFacilityDataItem]

    #[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
    pub input Input {
        /// Removes fluid from a storage.
        Fluid(reaction::input::Fluid<FluidPortSelector>),
        /// Removes heat from a storage.
        ///
        /// For reactors that consume coldness instead,
        /// they should use a catalyst and an output.
        Heat(reaction::input::Heat<FluidPortSelector>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
    pub catalyst Catalyst {
        Fluid(reaction::catalyst::Fluid<FluidPortSelector>),
        Pressure(reaction::catalyst::Pressure<FluidPortSelector>),
        Temperature(reaction::catalyst::Temperature<FluidPortSelector>),
        ResidentAttr(reaction::catalyst::ResidentAttr<ResidentSlotSelector>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
    pub output Output {
        Fluid(reaction::output::Fluid<FluidPortSelector>),
        Temperature(reaction::output::Heat<FluidPortSelector>),
        ResidentAttr(reaction::output::ResidentAttr<ResidentSlotSelector>),
    }
}
