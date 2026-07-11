use std::iter;

use bevy::app::{self, App, Plugin};
use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Query, Res, SystemParam};
use bevy::ecs::world::Mut;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::graph::facility;
use crate::util::QueryExt;
use crate::{fluid, reaction, resident};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.add_systems(app::FixedUpdate, interact_system.before(fluid::TransferSystemSet));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Resource)]
pub struct Interactions {
    pub list: Vec<Interaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Interaction {
    pub inputs:           Vec<Input>,
    pub catalysts:        Vec<Catalyst>,
    pub outputs:          Vec<Output>,
    pub time_step_period: u32,
    time_step_counter:    u32,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ResidentData {
    entity:     Entity,
    location:   &'static resident::Location,
    attributes: &'static mut resident::Attributes,
}

#[derive(SystemParam)]
struct InteractParams<'w, 's> {
    storage_query:  Query<'w, 's, &'static mut fluid::Storage>,
    facility_query: Query<'w, 's, &'static facility::OfBuilding>,
}

fn interact_system(
    mut resident_query: Query<ResidentData>,
    mut params: InteractParams,
    interactions: Res<Interactions>,
) {
    // TODO benchmark whether parallelizing this loop actually helps

    // TODO benchmark if loop resident/interaction or interaction/resident is better.
    // The former favors memory throughput, while the latter favors vectorization.

    for interaction in &interactions.list {
        for resident in &mut resident_query {
            interact_once(interaction, resident, &mut params);
        }
    }
}

fn interact_once(
    interaction: &Interaction,
    resident: ResidentDataItem,
    params: &mut InteractParams,
) -> Option<()> {
    let storage_entity = match *resident.location {
        resident::Location::Building { entity, .. }
        | resident::Location::Corridor { entity, .. } => entity,
        resident::Location::Facility { entity } => params.facility_query.log_get(entity)?.0,
    };
    let mut data = PreparedResidentData { data: resident, storage_entity };

    let _efficiency = execute_once(
        params,
        &mut data,
        &interaction.inputs,
        &interaction.catalysts,
        &interaction.outputs,
        1.0,
        1.0,
    );
    Some(())
}

struct PreparedResidentData<'dw, 'ds> {
    data:           ResidentDataItem<'dw, 'ds>,
    storage_entity: Entity,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct AmbientFluidSelector;

impl<'pw, 'ps, 'dw, 'ds>
    reaction::FluidStorageSelector<InteractParams<'pw, 'ps>, PreparedResidentData<'dw, 'ds>>
    for AmbientFluidSelector
{
    fn select<R>(
        &self,
        params: &InteractParams<'pw, 'ps>,
        data: &PreparedResidentData<'dw, 'ds>,
        then: impl FnOnce(&fluid::Storage) -> R,
    ) -> Option<R> {
        params.storage_query.log_get(data.storage_entity).map(then)
    }

    fn select_mut<R>(
        &self,
        params: &mut InteractParams<'pw, 'ps>,
        data: &mut PreparedResidentData<'dw, 'ds>,
        then: impl FnOnce(&mut fluid::Storage) -> R,
    ) -> Option<R> {
        params.storage_query.log_get_mut(data.storage_entity).map(|mut storage| then(&mut storage))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct SelfResidentSelector;

impl<'pw, 'ps, 'dw, 'ds>
    reaction::ResidentSelector<InteractParams<'pw, 'ps>, PreparedResidentData<'dw, 'ds>>
    for SelfResidentSelector
{
    fn for_each_attributes(
        &self,
        params: &InteractParams<'pw, 'ps>,
        data: &PreparedResidentData<'dw, 'ds>,
        mut then: impl FnMut(&resident::Attributes, Entity),
    ) {
        then(&data.data.attributes, data.data.entity);
    }

    fn for_each_attributes_mut(
        &self,
        params: &mut InteractParams<'pw, 'ps>,
        data: &mut PreparedResidentData<'dw, 'ds>,
        mut then: impl FnMut(&mut resident::Attributes, Entity),
    ) {
        then(&mut data.data.attributes, data.data.entity);
    }
}

reaction::define_ruleset! {
    [P = InteractParams, D = PreparedResidentData]
    fn execute_once;

    #[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
    pub input Input {
        Fluid(reaction::input::Fluid<AmbientFluidSelector>),
        Heat(reaction::input::Heat<AmbientFluidSelector>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
    pub catalyst Catalyst {
        Fluid(reaction::catalyst::Fluid<AmbientFluidSelector>),
        Temperature(reaction::catalyst::Temperature<AmbientFluidSelector>),
        ResidentAttr(reaction::catalyst::ResidentAttr<SelfResidentSelector>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
    pub output Output {
        Fluid(reaction::output::Fluid<AmbientFluidSelector>),
        Heat(reaction::output::Heat<AmbientFluidSelector>),
        ResidentAttr(reaction::output::ResidentAttr<SelfResidentSelector>),
    }
}
