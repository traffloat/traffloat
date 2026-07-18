use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::{QueryData, With};
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::system::{Query, SystemParam};
use bevy::reflect::Reflect;
use enum_map::EnumMap;
use serde::{Deserialize, Serialize};

use crate::util::QueryExt;
use crate::vehicle::{
    CompartmentList, CompartmentPassengerList, Location, OperatorList, OperatorOf, Vehicle,
};
use crate::{fluid, graph, reaction, resident};

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Propulsion {
    pub inputs:    Vec<Input>,
    pub catalysts: Vec<Catalyst>,
    pub outputs:   Vec<Output>,
}

#[derive(Component, Reflect, Default)]
pub struct Status {
    pub efficiency: f32,
    pub force:      f32,
}

#[derive(SystemParam)]
struct ExecuteParams<'w, 's> {
    fluid_storage_query: Query<'w, 's, &'static mut fluid::Storage>,
    conduit_query:       Query<'w, 's, &'static graph::conduit::OfCorridor>,
    compartment_query:   Query<'w, 's, &'static CompartmentPassengerList>,
    resident_query:      Query<'w, 's, ResidentData, With<resident::Resident>>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ExecuteData {
    entity:       Entity,
    location:     &'static Location,
    compartments: &'static CompartmentList,
    operators:    Option<&'static OperatorList>,
    status:       &'static mut Status,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ResidentData {
    as_operator: &'static OperatorOf,
    attributes:  &'static mut resident::Attributes,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum FluidStorageSelector {
    /// Ambient fluid outside the vehicle.
    Ambient,
    /// Fluid of the given compartment.
    Compartment(u32),
}

impl FluidStorageSelector {
    fn storage_entity(&self, params: &ExecuteParams, data: &ExecuteDataItem) -> Option<Entity> {
        Some(match *self {
            FluidStorageSelector::Ambient => match *data.location {
                Location::Building { building, .. } => building,
                Location::Rail { conduit, .. } => {
                    let corridor = params.conduit_query.log_get(conduit)?;
                    corridor.0
                }
            },
            FluidStorageSelector::Compartment(compartment) => {
                let &compartment_entity =
                    data.compartments.0.get(usize::try_from(compartment).expect("usize >= u32"))?;
                compartment_entity
            }
        })
    }
}

impl<'pw, 'ps, 'dw, 'ds>
    reaction::FluidStorageSelector<ExecuteParams<'pw, 'ps>, ExecuteDataItem<'dw, 'ds>>
    for FluidStorageSelector
{
    fn select<R>(
        &self,
        params: &ExecuteParams<'pw, 'ps>,
        data: &ExecuteDataItem<'dw, 'ds>,
        then: impl FnOnce(&fluid::Storage) -> R,
    ) -> Option<R> {
        let storage_entity = self.storage_entity(params, data)?;
        let storage = params.fluid_storage_query.log_get(storage_entity)?;
        Some(then(storage))
    }

    fn select_mut<R>(
        &self,
        params: &mut ExecuteParams<'pw, 'ps>,
        data: &mut ExecuteDataItem<'dw, 'ds>,
        then: impl FnOnce(&mut fluid::Storage) -> R,
    ) -> Option<R> {
        let storage_entity = self.storage_entity(params, data)?;
        let mut storage = params.fluid_storage_query.log_get_mut(storage_entity)?;
        Some(then(&mut storage))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum ResidentSelector {
    /// Only applies to operators of the given slot.
    Operator { slot: u32 },
    /// Only applies to passengers in the given compartment,
    /// including operators (who are also passengers).
    Passenger { compartment: u32 },
}

impl<'pw, 'ps, 'dw, 'ds>
    reaction::ResidentSelector<ExecuteParams<'pw, 'ps>, ExecuteDataItem<'dw, 'ds>>
    for ResidentSelector
{
    // These two functions only differ by mutability.
    // One uses ResidentDataItem while the other one uses ResidentDataReadOnlyItem,
    // which are difficult to merge with generics.

    fn for_each_attributes(
        &self,
        params: &ExecuteParams<'pw, 'ps>,
        data: &ExecuteDataItem<'dw, 'ds>,
        mut then: impl FnMut(&resident::Attributes, Entity),
    ) {
        match *self {
            Self::Operator { slot } => {
                let slot = usize::try_from(slot).expect("usize >= u32");
                for operator_entity in data.operators.iter().flat_map(|list| list.iter()) {
                    let Some(resident) = params.resident_query.log_get(operator_entity) else {
                        continue;
                    };
                    debug_assert_eq!(resident.as_operator.vehicle, data.entity);
                    if resident.as_operator.slot == slot {
                        then(resident.attributes, data.entity);
                    }
                }
            }
            Self::Passenger { compartment } => {
                let compartment = usize::try_from(compartment).expect("usize >= u32");
                let Some(&compartment_entity) = data.compartments.0.get(compartment) else {
                    return;
                };
                for passenger_entity in params
                    .compartment_query
                    .log_get(compartment_entity)
                    .iter()
                    .flat_map(|list| list.iter())
                {
                    let Some(resident) = params.resident_query.log_get(passenger_entity) else {
                        continue;
                    };
                    then(resident.attributes, data.entity);
                }
            }
        }
    }

    fn for_each_attributes_mut(
        &self,
        params: &mut ExecuteParams<'pw, 'ps>,
        data: &mut ExecuteDataItem<'dw, 'ds>,
        mut then: impl FnMut(&mut resident::Attributes, Entity),
    ) {
        match *self {
            Self::Operator { slot } => {
                let slot = usize::try_from(slot).expect("usize >= u32");
                for operator_entity in data.operators.iter().flat_map(|list| list.iter()) {
                    let Some(mut resident) = params.resident_query.log_get_mut(operator_entity)
                    else {
                        continue;
                    };
                    debug_assert_eq!(resident.as_operator.vehicle, data.entity);
                    if resident.as_operator.slot == slot {
                        then(&mut resident.attributes, data.entity);
                    }
                }
            }
            Self::Passenger { compartment } => {
                let compartment = usize::try_from(compartment).expect("usize >= u32");
                let Some(&compartment_entity) = data.compartments.0.get(compartment) else {
                    return;
                };
                for passenger_entity in params
                    .compartment_query
                    .log_get(compartment_entity)
                    .iter()
                    .flat_map(|list| list.iter())
                {
                    let Some(mut resident) = params.resident_query.log_get_mut(passenger_entity)
                    else {
                        continue;
                    };
                    then(&mut resident.attributes, data.entity);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ForceOutput {
    pub max_force: f32,
}

impl reaction::ReactionExecutor<ExecuteParams<'_, '_>, ExecuteDataItem<'_, '_>> for ForceOutput {
    fn execute(&self, efficiency: f32, params: &mut ExecuteParams, data: &mut ExecuteDataItem) {
        *data.status = Status { efficiency, force: self.max_force * efficiency };
    }
}

reaction::define_ruleset! {
    [P = ExecuteParams, D = ExecuteDataItem]

    #[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
    pub input Input {
        Fluid(reaction::input::Fluid<FluidStorageSelector>),
        Heat(reaction::input::Heat<FluidStorageSelector>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
    pub catalyst Catalyst {
        Fluid(reaction::catalyst::Fluid<FluidStorageSelector>),
        Pressure(reaction::catalyst::Pressure<FluidStorageSelector>),
        Temperature(reaction::catalyst::Temperature<FluidStorageSelector>),
        ResidentAttribute(reaction::catalyst::ResidentAttr<ResidentSelector>),
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
    pub output Output {
        Fluid(reaction::output::Fluid<FluidStorageSelector>),
        Heat(reaction::output::Heat<FluidStorageSelector>),
        Force(ForceOutput),
    }
}
