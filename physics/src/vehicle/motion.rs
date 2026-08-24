//! This module handles high-level vehicle motion in and between buildings and rails.
//! determining whether to accelerate or decelerate based on vehicle motion rules.
//!
//! In particular, this module is responsible for controlling braking
//! with the primary purpose to avoid collision.
//!
//! # Input
//! The driver resident AI executes pathfinding on its own,
//! generating instructions in the form of [`Intent`].
//!
//! # Output
//! The motion plugin updates [`super::propulsion::Desired`],
//! which is executed by the propulsion plugin to actually execute the motion plan.

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::{QueryData, With};
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, Query, Res, SystemParam};
use bevy::math::Vec3;
use bevy::reflect::Reflect;
use bevy::time::{self, Time};
use traffloat_proto::proto::AlphaOrBeta;

use crate::graph::{Corridor, conduit, edge};
use crate::util::{Alpha, Beta, InspectLog, QueryExt, Which};
use crate::vehicle::rail::ReservedDirection;
use crate::vehicle::{
    self, AttemptLocationTransitionCommand, Location, LocationBuilding, LocationRail, Rail,
    SystemSets, TypeDef, Vehicle, propulsion, rail,
};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Intent>();
        app.add_systems(app::FixedUpdate, control_system.in_set(SystemSets::Motion));
    }
}

/// Input for the motion plugin,
/// indicating the next step of motion to prepare for braking or reservation.
#[derive(Component, Reflect, Default, Debug, Clone, Copy)]
pub enum Intent {
    /// No defined intent, just stop and await decision.
    /// Happens when the vehicle has no capable driver.
    #[default]
    Stationary,
    /// Stop at a specific position inside a building.
    /// The vehicle must be currently in the building
    /// or in an adjacent rail heading towards the building.
    BuildingStop {
        /// The building to stop at.
        target:               Entity,
        /// The position inside the building to stop at.
        stop_at_interior_pos: Vec3,
    },
    /// Move towards a target rail.
    ///
    /// This intent must be immediately invalidated upon entering the target rail.
    EnterRail {
        /// The other rail to move towards.
        /// This should not be the rail the vehicle is currently on;
        /// if it is, the intent is interpreted as
        /// moving to `through_building` then bouncing back.
        target_rail:      Entity,
        /// If the vehicle is currently in a building,
        /// this must be the building that the vehicle is currently in.
        ///
        /// If the vehicle is currently on a rail,
        /// this field serves as a hint to determine which direction to move towards.
        /// This must be one of the two endpoint buildings of the current rail,
        /// and the target rail must be adjacent to this building.
        through_building: Entity,
    },
    // TODO support moving to end of rail for construction vehicles to construct building?
}

#[derive(SystemParam)]
struct ControlVehicleParams<'w, 's> {
    conduit_query:  Query<'w, 's, ControlConduitData>,
    corridor_query: Query<'w, 's, ControlCorridorData, With<Corridor>>,
    edge_alpha:     EdgeParams<'w, 's, Alpha>,
    edge_beta:      EdgeParams<'w, 's, Beta>,
    vehicle_query:  Query<'w, 's, (&'static Vehicle, &'static Location)>,
    types:          Res<'w, super::Types>,
    config:         Res<'w, super::Conf>,
}

#[derive(SystemParam)]
struct EdgeParams<'w, 's, Ab: Which> {
    edge_building_query: Query<'w, 's, &'static edge::OfBuilding<Ab>>,
    edge_corridor_query: Query<'w, 's, &'static edge::OfCorridor<Ab>>,
    building_query:      Query<'w, 's, &'static edge::BuildingEdges<Ab>>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct ControlVehicleData {
    entity:   Entity,
    vehicle:  &'static Vehicle,
    intent:   &'static mut Intent,
    location: &'static Location,
}

#[derive(QueryData)]
struct ControlConduitData {
    corridor:    &'static conduit::OfCorridor,
    rail:        &'static Rail,
    reservation: &'static rail::Reservation,
    vehicles:    Option<&'static vehicle::ListOnRail>,
}

#[derive(QueryData)]
struct ControlCorridorData {
    entity:     Entity,
    corridor:   &'static Corridor,
    edge_alpha: Option<&'static edge::CorridorEdge<Alpha>>,
    edge_beta:  Option<&'static edge::CorridorEdge<Beta>>,
}

fn control_system(
    mut vehicle_query: Query<(ControlVehicleData, &mut propulsion::Desired)>,
    params: ControlVehicleParams,
    mut commands: Commands,
    time: Res<Time<time::Fixed>>,
) {
    let dt = time.delta_secs();
    vehicle_query
        .iter_mut()
        .for_each(|(data, mut desired)| *desired = control_once(data, &params, &mut commands, dt));
}

fn control_once(
    mut data: ControlVehicleDataItem,
    params: &ControlVehicleParams,
    commands: &mut Commands,
    dt: f32,
) -> propulsion::Desired {
    match (*data.location, *data.intent) {
        (_, Intent::Stationary) => propulsion::Desired::Stationary,
        (Location::Building(location), Intent::BuildingStop { target, stop_at_interior_pos }) => {
            control_on_rail_building_local(
                data.entity,
                location,
                target,
                stop_at_interior_pos,
                dt,
                &mut data.intent,
            )
        }
        (
            Location::Rail(location),
            Intent::EnterRail { target_rail: next, through_building: thru },
        ) => {
            // move from `curr` to `next` through `thru`,
            // check if inertial motion is possible through `thru` to `next`,
            // otherwise, slow down to std drifting speed before reaching `thru`,
            // transitioning if the building is within the current speed interval within `dt`
            control_on_rail(
                ControlOnRail {
                    vehicle_entity: data.entity,
                    vehicle: data.vehicle,
                    location,
                    building: thru,
                    inertial_target: Some(next),
                    dt,
                },
                params,
                commands,
            )
            .unwrap_or(propulsion::Desired::Stationary)
        }
        (Location::Rail(location), Intent::BuildingStop { target: building, .. }) => {
            // move from `conduit` and slow down to std drifting speed before reaching `target`,
            // transitioning if the building is within the current speed interval within `dt`
            control_on_rail(
                ControlOnRail {
                    vehicle_entity: data.entity,
                    vehicle: data.vehicle,
                    location,
                    building,
                    inertial_target: None,
                    dt,
                },
                params,
                commands,
            )
            .unwrap_or(propulsion::Desired::Stationary)
        }
        (
            Location::Building(location),
            Intent::EnterRail { target_rail: rail, through_building: thru },
        ) => {
            // move towards the edge location for `rail`,
            // transitioning if the edge is within the std drifting sphere within `dt`
            debug_assert_eq!(location.building, thru);

            let Some(rail_data) = params.conduit_query.log_get(rail) else {
                return propulsion::Desired::Stationary;
            };
            let corridor = rail_data.corridor.0;
            let args = BuildingToRailArgs { location, corridor, rail, dt };

            control_building_to_rail(params, &params.edge_alpha, &data, commands, &args)
                .or_else(|| {
                    control_building_to_rail(params, &params.edge_beta, &data, commands, &args)
                })
                .unwrap_or_else(|| {
                    tracing::warn!(
                        "Vehicle {:?} received invalid motion intent, is in building {:?}, which \
                         is not connected to the intent target rail {rail:?}",
                        data.entity,
                        location.building,
                    );
                    propulsion::Desired::Stationary
                })
        }
    }
}

fn control_on_rail_building_local(
    vehicle_entity: Entity,
    location: LocationBuilding,
    target: Entity,
    stop_at_interior_pos: Vec3,
    dt: f32,
    desired: &mut Intent,
) -> propulsion::Desired {
    if location.building != target {
        tracing::warn!(
            "Vehicle {vehicle_entity:?} received invalid motion intent, is in building {:?} but \
             intent target is {target:?}",
            location.building,
        );
        return propulsion::Desired::Stationary;
    }

    if location.interior_pos.distance_squared(stop_at_interior_pos)
        < location.speed.length_squared() * dt.powi(2)
    {
        // intent fulfilled, clear intent
        *desired = Intent::Stationary;
    }

    propulsion::Desired::Building { interior_pos: stop_at_interior_pos }
}

struct BuildingToRailArgs {
    location: LocationBuilding,
    corridor: Entity,
    rail:     Entity,
    dt:       f32,
}

fn control_building_to_rail<Ab: Which>(
    params: &ControlVehicleParams,
    edge_params: &EdgeParams<Ab>,
    vehicle_data: &ControlVehicleDataItem,
    commands: &mut Commands,
    args: &BuildingToRailArgs,
) -> Option<propulsion::Desired> {
    let edge_interior_pos = find_edge_interior_pos_to_corridor(
        &edge_params.building_query,
        &edge_params.edge_corridor_query,
        &edge_params.edge_building_query,
        args.location.building,
        args.corridor,
    )?;

    let distance = edge_interior_pos.distance_squared(args.location.interior_pos);
    let half_vehicle_length = params.types.get(vehicle_data.vehicle.ty).physical.length * 0.5;

    if distance < half_vehicle_length.powi(2) {
        let corridor_length = params.corridor_query.log_get(args.corridor)?.corridor.length;

        commands.entity(vehicle_data.entity).queue(AttemptLocationTransitionCommand {
            new_location: Location::Rail(LocationRail {
                rail:                args.rail,
                distance_from_alpha: Ab::default()
                    .select_with(-half_vehicle_length, corridor_length + half_vehicle_length),
                speed_from_alpha:    Ab::default()
                    .negate_if_beta(params.config.standard_drifting_speed),
            }),
            entry_method: Ab::default(),
        });
    }

    Some(propulsion::Desired::Building { interior_pos: edge_interior_pos })
}

fn find_edge_interior_pos_to_corridor<Ab: Which>(
    building_query: &Query<&edge::BuildingEdges<Ab>>,
    edge_corridor_query: &Query<&edge::OfCorridor<Ab>>,
    edge_building_query: &Query<&edge::OfBuilding<Ab>>,
    building: Entity,
    corridor: Entity,
) -> Option<Vec3> {
    let edges = building_query.get(building).ok()?;
    edges
        .iter()
        .find(|&edge| {
            edge_corridor_query.log_get(edge).is_some_and(|of_corridor| of_corridor.0 == corridor)
        })
        .and_then(|edge| edge_building_query.log_get(edge))
        .map(|of_building| of_building.interior_pos)
}

struct ControlOnRail<'q> {
    vehicle_entity:  Entity,
    vehicle:         &'q Vehicle,
    location:        LocationRail,
    building:        Entity,
    /// Next rail to move onto, for inertial motion.
    inertial_target: Option<Entity>,
    dt:              f32,
}

fn control_on_rail(
    args: ControlOnRail,
    params: &ControlVehicleParams,
    commands: &mut Commands,
) -> Option<propulsion::Desired> {
    let def = params.types.get(args.vehicle.ty);

    let conduit = params.conduit_query.log_get(args.location.rail)?;
    let corridor = params.corridor_query.log_get(conduit.corridor.0)?;
    let Some(exit_endpoint) = identify_endpoint(
        &params.edge_alpha.edge_building_query,
        corridor.edge_alpha,
        args.building,
    )
    .or_else(|| {
        identify_endpoint(&params.edge_beta.edge_building_query, corridor.edge_beta, args.building)
    }) else {
        tracing::warn!(
            "{:?} not connected to the current location (rail {:?}) of vehicle {:?}",
            args.building,
            args.location.rail,
            args.vehicle_entity,
        );
        return None;
    };

    control_on_rail_check_current_reservation(&conduit, &args, exit_endpoint).ok()?;

    let try_transition = match exit_endpoint {
        AlphaOrBeta::Alpha => {
            control_on_rail_try_transition_to_building(params, &args, &corridor, Alpha, commands)
        }
        AlphaOrBeta::Beta => {
            control_on_rail_try_transition_to_building(params, &args, &corridor, Beta, commands)
        }
    };
    if try_transition {
        return None;
    }

    if let Some(result) = control_on_rail_try_acquire_inertial(
        params,
        &args,
        &corridor,
        &params.edge_alpha.edge_building_query,
    ) {
        return Some(result);
    }

    if let Some(result) = control_on_rail_try_acquire_inertial(
        params,
        &args,
        &corridor,
        &params.edge_beta.edge_building_query,
    ) {
        return Some(result);
    }

    let vehicle_list =
        conduit.vehicles.inspect_log("rail conduit owning vehicle must have vehicle list")?;
    let index_in_list = vehicle_list
        .iter()
        .position(|v| v == args.vehicle_entity)
        .inspect_log("rail conduit owning vehicle must contain vehicle in list")?;
    let next_vehicle = match exit_endpoint {
        AlphaOrBeta::Alpha => {
            index_in_list.checked_sub(1).and_then(|minus_one| vehicle_list.get(minus_one))
        }
        AlphaOrBeta::Beta => vehicle_list.get(index_in_list + 1),
    };
    match next_vehicle {
        Some(value) => control_on_rail_with_vehicle_stop(
            params,
            def,
            conduit.rail,
            &args,
            exit_endpoint,
            value,
        ),
        None => Some(control_on_rail_with_exit_stop(
            params,
            def,
            corridor.corridor,
            conduit.rail,
            &args,
            exit_endpoint,
        )),
    }
}

fn control_on_rail_check_current_reservation(
    conduit: &ControlConduitDataItem,
    args: &ControlOnRail,
    exit_endpoint: AlphaOrBeta,
) -> Result<(), ()> {
    match conduit.reservation.inner {
        None => {
            tracing::warn!(
                "Vehicle {:?} is on an unreserved rail {:?}",
                args.vehicle_entity,
                args.location.rail,
            );
            Err(())
        }
        Some(ref inner) if inner.direction != ReservedDirection::from_exit(exit_endpoint) => {
            tracing::warn!(
                "Vehicle {:?} is on rail {:?} with unexpected reserved direction {:?}",
                args.vehicle_entity,
                args.location.rail,
                inner.direction,
            );
            Err(())
        }
        Some(_) => Ok(()),
    }
}

fn control_on_rail_try_transition_to_building(
    params: &ControlVehicleParams,
    args: &ControlOnRail,
    corridor: &ControlCorridorDataItem,
    exit: impl Which,
    commands: &mut Commands,
) -> bool {
    fn find_interior_pos<Ab: Which>(
        corridor_edge: Option<&edge::CorridorEdge<Ab>>,
        edge_building_query: &Query<&edge::OfBuilding<Ab>>,
    ) -> Option<Vec3> {
        corridor_edge
            .and_then(|edge| edge_building_query.log_get(edge.edge()))
            .map(|edge| edge.interior_pos)
    }

    let def = params.types.get(args.vehicle.ty);

    let probe_speed =
        args.location.speed_from_alpha.abs().max(params.config.standard_drifting_speed);
    let probe_distance = probe_speed * args.dt;
    let distance_from_exit = exit.select_with(
        args.location.distance_from_alpha,
        corridor.corridor.length - args.location.distance_from_alpha,
    ) + def.physical.length * 0.5;

    if distance_from_exit > probe_distance {
        return false;
    }

    let Some(interior_pos) = exit.select_lazy(
        || find_interior_pos(corridor.edge_alpha, &params.edge_alpha.edge_building_query),
        || find_interior_pos(corridor.edge_beta, &params.edge_beta.edge_building_query),
    ) else {
        tracing::error!(
            "cannot find interior pos from corridor {:?} to building {:?}",
            corridor.entity,
            args.building
        );
        return false;
    };
    commands.entity(args.vehicle_entity).queue(AttemptLocationTransitionCommand {
        new_location: Location::Building(LocationBuilding {
            building: args.building,
            interior_pos,
            speed: Vec3::ZERO,
        }),
        entry_method: exit,
    });

    true
}

fn control_on_rail_try_acquire_inertial<Exit: Which>(
    params: &ControlVehicleParams,
    args: &ControlOnRail,
    corridor: &ControlCorridorDataItem,
    edge_building_query: &Query<&edge::OfBuilding<Exit>>,
) -> Option<propulsion::Desired> {
    None // TODO
}

fn control_on_rail_with_vehicle_stop(
    params: &ControlVehicleParams,
    def: &TypeDef,
    rail: &Rail,
    args: &ControlOnRail,
    exit_endpoint: AlphaOrBeta,
    next_entity: Entity,
) -> Option<propulsion::Desired> {
    let (next_vehicle_data, &Location::Rail(next_location)) =
        params.vehicle_query.log_get(next_entity)?
    else {
        tracing::warn!(
            "rail conduit owning vehicle {:?} must have next vehicle {:?} on the same rail",
            args.vehicle_entity,
            next_entity
        );
        return None;
    };
    let next_length = params.types.get(next_vehicle_data.ty).physical.length;

    let padding = next_length.midpoint(def.physical.length) + params.config.safety_headroom;

    let next_displace = next_location.distance_from_alpha;
    let distance = match exit_endpoint {
        AlphaOrBeta::Alpha => (args.location.distance_from_alpha - next_displace) - padding,
        AlphaOrBeta::Beta => (next_displace - args.location.distance_from_alpha) - padding,
    };

    Some(control_on_rail_with_stop(
        def,
        args.vehicle,
        rail,
        exit_endpoint,
        distance,
        params.config.reaction_time.as_secs_f32(),
        0.0, // full stop due to vehicle ahead
    ))
}

fn control_on_rail_with_exit_stop(
    params: &ControlVehicleParams,
    def: &TypeDef,
    corridor: &Corridor,
    rail: &Rail,
    args: &ControlOnRail,
    exit_endpoint: AlphaOrBeta,
) -> propulsion::Desired {
    let distance_from_exit = match exit_endpoint {
        AlphaOrBeta::Alpha => args.location.distance_from_alpha,
        AlphaOrBeta::Beta => corridor.length - args.location.distance_from_alpha,
    };

    let distance = distance_from_exit + def.physical.length * 0.5;
    control_on_rail_with_stop(
        def,
        args.vehicle,
        rail,
        exit_endpoint,
        distance,
        0.0,
        params.config.standard_drifting_speed,
    )
}

fn control_on_rail_with_stop(
    def: &TypeDef,
    vehicle: &Vehicle,
    rail: &Rail,
    exit_endpoint: AlphaOrBeta,
    safety_distance: f32,
    reaction_time: f32,
    terminal_speed: f32,
) -> propulsion::Desired {
    let max_speed = max_stoppable_speed(
        safety_distance,
        def.motion.max_braking / vehicle.mass,
        reaction_time,
        terminal_speed,
    )
    .min(def.motion.max_speed)
    .min(rail.max_speed)
    .max(0.0);

    propulsion::Desired::Rail {
        speed_from_alpha: match exit_endpoint {
            AlphaOrBeta::Alpha => -max_speed,
            AlphaOrBeta::Beta => max_speed,
        },
    }
}

fn identify_endpoint<Ab: Which>(
    edge_building_query: &Query<&edge::OfBuilding<Ab>>,
    edge: Option<&edge::CorridorEdge<Ab>>,
    building: Entity,
) -> Option<AlphaOrBeta> {
    let edge = edge?.edge();
    let of_building = edge_building_query.log_get(edge)?;
    if of_building.building == building { Some(Ab::default().proto()) } else { None }
}

fn max_stoppable_speed(
    distance: f32,
    braking: f32,
    reaction_time: f32,
    terminal_speed: f32,
) -> f32 {
    // distance before reaction: s1 = u*t
    // distance after reaction: v^2 = u^2 - 2*a*s2 => s2 = (u^2 - v^2) / (2*a)
    // s := s1 + s2 = u*t + (u^2 - v^2) / (2*a)
    // => (1/(2*a)) * u^2 + t * u + (-v^2)/(2*a) - s = 0
    // => u = sqrt((a*t)^2 + 2*a*s + v^2) - a*t

    ((braking * reaction_time).powi(2) + 2.0 * braking * distance + terminal_speed.powi(2)).sqrt()
        - braking * reaction_time
}

#[cfg(test)]
mod tests;
