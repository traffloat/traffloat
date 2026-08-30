use bevy::app::{self, App, Plugin};
use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Query, ResMut, SystemParam};
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};
use traffloat_util::QueryExt;

use crate::graph::facility;
use crate::persist::AppExt;
use crate::{fluid, reaction, resident};

mod persist;
pub use persist::Persist;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Interactions>();

        app.init_resource::<Interactions>();
        app.register_persistable(Persist);

        app.add_systems(
            app::FixedUpdate,
            interact_system.in_set(fluid::ModifySystemSets::ResidentAmbient),
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Resource, Default)]
pub struct Interactions {
    pub list: Vec<Interaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Interaction {
    pub name:             String,
    pub inputs:           Vec<Input>,
    pub catalysts:        Vec<Catalyst>,
    pub outputs:          Vec<Output>,
    pub time_step_period: u32,
    time_step_counter:    u32,
}

impl Interaction {
    pub fn new(name: impl Into<String>, time_step_period: u32) -> Self {
        Self {
            name: name.into(),
            inputs: Vec::new(),
            catalysts: Vec::new(),
            outputs: Vec::new(),
            time_step_period,
            time_step_counter: 0,
        }
    }

    #[must_use]
    pub fn with_input(mut self, input: Input) -> Self {
        self.inputs.push(input);
        self
    }

    #[must_use]
    pub fn with_catalyst(mut self, catalyst: Catalyst) -> Self {
        self.catalysts.push(catalyst);
        self
    }

    #[must_use]
    pub fn with_output(mut self, output: Output) -> Self {
        self.outputs.push(output);
        self
    }
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
    mut interactions: ResMut<Interactions>,
) {
    // TODO benchmark whether parallelizing this loop actually helps

    // TODO benchmark if loop resident/interaction or interaction/resident is better.
    // The former favors memory throughput, while the latter favors vectorization and branch
    // prediction.

    for interaction in &mut interactions.list {
        interaction.time_step_counter += 1;
        if interaction.time_step_counter == interaction.time_step_period {
            interaction.time_step_counter = 0;

            for resident in &mut resident_query {
                interact_once(interaction, resident, &mut params);
            }
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
        resident::Location::Vehicle { compartment } => compartment,
    };
    let mut data = PreparedResidentData { data: resident, storage_entity };

    let _efficiency = reaction::execute_once(
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

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Reflect, Default)]
pub struct SelfResidentSelector;

impl<'pw, 'ps, 'dw, 'ds>
    reaction::ResidentSelector<InteractParams<'pw, 'ps>, PreparedResidentData<'dw, 'ds>>
    for SelfResidentSelector
{
    fn for_each_attributes(
        &self,
        _: &InteractParams<'pw, 'ps>,
        data: &PreparedResidentData<'dw, 'ds>,
        mut then: impl FnMut(&resident::Attributes, Entity),
    ) {
        then(&data.data.attributes, data.data.entity);
    }

    fn for_each_attributes_mut(
        &self,
        _: &mut InteractParams<'pw, 'ps>,
        data: &mut PreparedResidentData<'dw, 'ds>,
        mut then: impl FnMut(&mut resident::Attributes, Entity),
    ) {
        then(&mut data.data.attributes, data.data.entity);
    }
}

reaction::define_ruleset! {
    [P = InteractParams, D = PreparedResidentData]

    #[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
    pub input Input {
        Fluid(reaction::input::Fluid<AmbientFluidSelector>),
        Heat(reaction::input::Heat<AmbientFluidSelector>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
    pub catalyst Catalyst {
        Fluid(reaction::catalyst::Fluid<AmbientFluidSelector>),
        Temperature(reaction::catalyst::Temperature<AmbientFluidSelector>),
        Pressure(reaction::catalyst::Pressure<AmbientFluidSelector>),
        ResidentAttr(reaction::catalyst::ResidentAttr<SelfResidentSelector>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
    pub output Output {
        Fluid(reaction::output::Fluid<AmbientFluidSelector>),
        Heat(reaction::output::Heat<AmbientFluidSelector>),
        ResidentAttr(reaction::output::ResidentAttr<SelfResidentSelector>),
    }
}
