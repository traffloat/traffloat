use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::{QueryData, With};
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Query, Res, SystemParam};
use bevy::math::Vec3;
use bevy::reflect::Reflect;
use bevy::time::{self, Time};
use enum_map::EnumMap;
use serde::{Deserialize, Serialize};

use crate::util::QueryExt;
use crate::vehicle::{
    CompartmentList, CompartmentPassengerList, Location, OperatorList, OperatorOf, SystemSets,
    TypeDef, Types, Vehicle,
};
use crate::{fluid, graph, reaction, resident};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Desired>();
        app.register_type::<Status>();
        app.add_systems(app::FixedUpdate, control_system.in_set(SystemSets::Propulsion));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Propulsion {
    pub inputs:    Vec<Input>,
    pub catalysts: Vec<Catalyst>,
    pub outputs:   Vec<Output>,
}

/// The desired speed of the vehicle, when on a rail.
#[derive(Component, Reflect, Default, Debug)]
pub enum Desired {
    #[default]
    Stationary,
    Building {
        interior_pos: Vec3,
    },
    Rail {
        speed_from_alpha: f32,
    },
}

#[derive(Component, Reflect, Default)]
#[require(Desired)]
pub struct Status {
    pub propulsion_efficiency: f32,
    pub propulsion_force:      f32,
    pub drag_force:            f32,
    pub brake_force:           f32,
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
    location:     &'static mut Location,
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
    /// Maximum force output of the propulsion system.
    ///
    /// Must be positive.
    pub max_force: f32,
}

impl reaction::ReactionExecutor<ExecuteParams<'_, '_>, ExecuteDataItem<'_, '_>> for ForceOutput {
    fn execute(&self, efficiency: f32, params: &mut ExecuteParams, data: &mut ExecuteDataItem) {
        data.status.propulsion_efficiency = efficiency;
        data.status.propulsion_force = self.max_force * efficiency;
    }

    fn execute_zero(&self, params: &mut ExecuteParams<'_, '_>, data: &mut ExecuteDataItem<'_, '_>) {
        data.status.propulsion_efficiency = 0.0;
        data.status.propulsion_force = 0.0;
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

#[derive(QueryData)]
#[query_data(mutable)]
struct ControlData {
    entity:  Entity,
    vehicle: &'static Vehicle,
    desired: &'static Desired,
    execute: ExecuteData,
}

#[derive(SystemParam)]
struct ControlParams<'w, 's> {
    execute: ExecuteParams<'w, 's>,
    config:  Res<'w, super::Conf>,
    types:   Res<'w, Types>,
}

fn control_system(
    time: Res<Time<time::Fixed>>,
    mut vehicle_query: Query<ControlData>,
    mut params: ControlParams,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 || dt.is_subnormal() {
        return;
    }

    vehicle_query.iter_mut().for_each(|data| control_once(data, &mut params, dt));
}

fn control_once(mut data: ControlDataItem, params: &mut ControlParams, dt: f32) {
    match (data.desired, &mut *data.execute.location) {
        (Desired::Stationary, &mut Location::Building { ref mut speed, .. }) => {
            *speed = Vec3::ZERO;
        }
        (
            &Desired::Building { interior_pos: desired_pos },
            &mut Location::Building { interior_pos: ref mut actual_pos, ref mut speed, .. },
        ) => {
            let (direction, dist) = (*actual_pos - desired_pos).normalize_and_length();
            let max_dist = params.config.standard_drifting_speed * dt;
            if dist > max_dist {
                *speed = direction * params.config.standard_drifting_speed;
                *actual_pos += *speed * dt;
            } else {
                *speed = direction * (dist / dt);
                *actual_pos = desired_pos;
            }
        }
        (
            Desired::Rail { .. } | Desired::Stationary,
            &mut Location::Rail {
                speed_from_alpha: ref mut actual_speed,
                conduit,
                distance_from_alpha: _,
            },
        ) => {
            let desired_speed = match *data.desired {
                Desired::Rail { speed_from_alpha } => speed_from_alpha,
                Desired::Stationary => 0.0,
                Desired::Building { .. } => unreachable!(),
            };

            let def = params.types.get(data.vehicle.ty);

            // Apply drag to the actual speed first, we will compensate for this through propulsion later if we can.
            apply_pressure(
                &params.execute,
                data.vehicle,
                &mut data.execute.status.drag_force,
                actual_speed,
                conduit,
                def,
                dt,
            );

            // Apply braking if desired speed is not greater than actual speed in the same direction
            apply_brake(
                desired_speed,
                &mut data.execute.status.brake_force,
                actual_speed,
                data.vehicle,
                def,
                dt,
            );

            // If the desired speed is greater than the current speed in the same direction,
            // apply propulsion to accelerate.
            let new_speed = apply_propulsion(
                data.vehicle,
                desired_speed,
                *actual_speed,
                def,
                dt,
                &mut params.execute,
                &mut data.execute,
            );
            // borrow again because apply_propulsion needs to read Location
            let Location::Rail {
                speed_from_alpha: actual_speed,
                distance_from_alpha: displacement,
                ..
            } = &mut *data.execute.location
            else {
                unreachable!("apply_propulsion should not change location type")
            };

            *actual_speed = new_speed;
            *displacement += *actual_speed * dt;
        }
        _ => {
            *data.execute.status = Status::default();
            tracing::warn!(
                "Unexpected combination of desired motion and actual location for vehicle {:?}: \
                 desired = {:?}, actual = {:?}",
                data.entity,
                data.desired,
                data.execute.location
            );
        }
    }
}

fn apply_pressure(
    params: &ExecuteParams,
    vehicle: &Vehicle,
    force: &mut f32,
    speed: &mut f32,
    conduit: Entity,
    def: &TypeDef,
    dt: f32,
) {
    let Some(corridor) = params.conduit_query.log_get(conduit) else { return };
    let pressure =
        params.fluid_storage_query.log_get(corridor.0).map_or(0.0, |storage| storage.pressure);
    *force = def.motion.drag_coefficient * pressure * speed.powi(2);
    let drag_v_delta = *force / vehicle.mass * dt;
    *speed = if *speed > 0.0 {
        (*speed - drag_v_delta).max(0.0)
    } else {
        (*speed + drag_v_delta).min(0.0)
    };
}

fn apply_brake(
    desired_speed: f32,
    force: &mut f32,
    actual_speed: &mut f32,
    vehicle: &Vehicle,
    def: &TypeDef,
    dt: f32,
) {
    let braking_target_abs =
        if desired_speed == 0.0 || desired_speed.signum() != actual_speed.signum() {
            Some(0.0)
        } else if desired_speed.abs() < actual_speed.abs() {
            Some(desired_speed.abs())
        } else {
            None
        };
    if let Some(braking_target_abs) = braking_target_abs {
        let max_braking = def.motion.max_braking / vehicle.mass;
        let max_v_delta = max_braking * dt;

        if actual_speed.abs() - braking_target_abs > max_v_delta {
            *actual_speed -= max_v_delta * actual_speed.signum();
            *force = def.motion.max_braking;
        } else {
            *actual_speed = braking_target_abs * actual_speed.signum();
            *force =
                def.motion.max_braking * (actual_speed.abs() - braking_target_abs) / max_v_delta;
        }
    } else {
        *force = 0.0;
    }
}

fn apply_propulsion(
    vehicle: &Vehicle,
    desired_speed: f32,
    actual_speed: f32,
    def: &TypeDef,
    dt: f32,
    params: &mut ExecuteParams,
    data: &mut ExecuteDataItem,
) -> f32 {
    if actual_speed * desired_speed > 0.0 && actual_speed.abs() > desired_speed.abs() {
        // Only braking required, no more propulsion to exert.
        data.status.propulsion_efficiency = 0.0;
        data.status.propulsion_force = 0.0;
        return actual_speed;
    }

    let required_force = (desired_speed - actual_speed).abs() * vehicle.mass / dt;

    let propulsion = &def.motion.propulsion;
    let max_output_force = propulsion
        .outputs
        .iter()
        .map(|output| match output {
            Output::Force(force) => force.max_force,
            _ => 0.0,
        })
        .sum::<f32>();
    let max_efficiency = (required_force / max_output_force).min(1.0);

    let efficiency = reaction::execute_once(
        params,
        data,
        &propulsion.inputs,
        &propulsion.catalysts,
        &propulsion.outputs,
        max_efficiency,
        1.0,
    );

    let acceleration = data.status.propulsion_force * desired_speed.signum() / vehicle.mass;
    actual_speed + acceleration * dt
}

#[cfg(test)]
mod tests;
