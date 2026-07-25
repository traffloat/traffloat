//! This module handles high-level vehicle motion in and between buildings and rails.
//! determining whether to accelerate or deceelerate based on vehicle motion rules.
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
use bevy::ecs::query::{AnyOf, QueryData, With};
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Query, Res, SystemParam};
use bevy::math::Vec3;
use bevy::reflect::Reflect;
use traffloat_proto::proto::AlphaOrBeta;

use crate::graph::{Building, Corridor, conduit, edge};
use crate::util::{Alpha, Beta, InspectLog, QueryExt, Which};
use crate::vehicle::rail::ReservedDirection;
use crate::vehicle::{self, Location, Rail, SystemSets, TypeDef, Vehicle, propulsion, rail};

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
    #[default]
    None,
    Building {
        target:               Entity,
        stop_at_interior_pos: Vec3,
    },
    Rail {
        target_rail:      Entity,
        through_building: Entity,
    },
}

#[derive(SystemParam)]
struct ControlVehicleParams<'w, 's> {
    conduit_query:          Query<'w, 's, ControlConduitData>,
    corridor_query:         Query<'w, 's, ControlCorridorData, With<Corridor>>,
    edge_query:
        Query<'w, 's, AnyOf<(&'static edge::OfBuilding<Alpha>, &'static edge::OfBuilding<Beta>)>>,
    building_query:         Query<'w, 's, &'static Building>,
    vehicle_location_query: Query<'w, 's, &'static Location>,
    types:                  Res<'w, super::Types>,
    config:                 Res<'w, super::Conf>,
}

#[derive(QueryData)]
struct ControlVehicleData {
    entity:   Entity,
    vehicle:  &'static Vehicle,
    intent:   &'static Intent,
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
    corridor:    &'static Corridor,
    edges_alpha: Option<&'static edge::CorridorEdge<Alpha>>,
    edges_beta:  Option<&'static edge::CorridorEdge<Beta>>,
}

fn control_system(
    mut vehicle_query: Query<(ControlVehicleData, &mut propulsion::Desired)>,
    params: ControlVehicleParams,
) {
    vehicle_query.iter_mut().for_each(|(data, mut desired)| *desired = control_once(data, &params));
}

fn control_once(
    data: ControlVehicleDataItem,
    params: &ControlVehicleParams,
) -> propulsion::Desired {
    match (*data.location, *data.intent) {
        (_, Intent::None) => propulsion::Desired::Stationary,
        (
            Location::Building { building, .. },
            Intent::Building { target, stop_at_interior_pos },
        ) => {
            if building == target {
                propulsion::Desired::Building { interior_pos: stop_at_interior_pos }
            } else {
                tracing::warn!(
                    "Vehicle {:?} received invalid motion intent, is in building {building:?} but \
                     intent target is {target:?}",
                    data.entity
                );
                propulsion::Desired::Stationary
            }
        }
        (
            Location::Rail {
                conduit: curr,
                distance_from_alpha: displace,
                speed_from_alpha: current_speed,
            },
            Intent::Rail { target_rail: next, through_building: thru },
        ) => {
            // move from `curr` to `next` through `thru`,
            // check if inertial motion is possible through `thru` to `next`,
            // otherwise, slow down to std drifting speed before reaching `thru`,
            // transitioning if the building is within the current speed interval within `dt`

            // TODO inertial motion
            // for now assume we always need to slow down to std drifting speed

            control_on_rail(
                ControlOnRail {
                    vehicle_entity: data.entity,
                    vehicle: data.vehicle,
                    rail: curr,
                    displace,
                    current_speed,
                    building: thru,
                },
                params,
            )
            .unwrap_or(propulsion::Desired::Stationary)
        }
        (
            Location::Rail {
                conduit: rail,
                distance_from_alpha: displace,
                speed_from_alpha: current_speed,
            },
            Intent::Building { target: building, .. },
        ) => {
            // move from `conduit` and slow down to std drifting speed before reaching `target`,
            // transitioning if the building is within the current speed interval within `dt`

            control_on_rail(
                ControlOnRail {
                    vehicle_entity: data.entity,
                    vehicle: data.vehicle,
                    rail,
                    displace,
                    current_speed,
                    building,
                },
                params,
            )
            .unwrap_or(propulsion::Desired::Stationary)
        }
        (
            Location::Building { building, .. },
            Intent::Rail { target_rail: rail, through_building: thru },
        ) => {
            // move towards the edge location for `rail`,
            // transitioning if the edge is within the std drifting sphere within `dt`
            todo!()
        }
    }
}

struct ControlOnRail<'q> {
    vehicle_entity: Entity,
    vehicle:        &'q Vehicle,
    rail:           Entity,
    /// Distance of the center of the vehicle from the alpha end of the corridor.
    displace:       f32,
    current_speed:  f32,
    building:       Entity,
}

fn control_on_rail(
    args: ControlOnRail,
    params: &ControlVehicleParams,
    // TODO inertial
) -> Option<propulsion::Desired> {
    enum NextStop {
        Vehicle(Entity),
        Endpoint(f32),
    }

    let def = params.types.get(args.vehicle.ty);

    let conduit = params.conduit_query.log_get(args.rail)?;
    let corridor = params.corridor_query.log_get(conduit.corridor.0)?;
    let Some(exit_endpoint) =
        identify_endpoint(params, corridor.edges_alpha, args.building, |(a, _)| a)
            .or_else(|| identify_endpoint(params, corridor.edges_beta, args.building, |(_, b)| b))
    else {
        tracing::warn!(
            "through_building {:?} not connected to the current location (rail {:?}) of vehicle \
             {:?}",
            args.building,
            args.rail,
            args.vehicle_entity
        );
        return None;
    };

    match conduit.reservation.inner {
        None => {
            tracing::warn!(
                "Vehicle {:?} is on an unreserved rail {:?}",
                args.vehicle_entity,
                args.rail
            );
            return None;
        }
        Some(ref inner) if inner.direction != ReservedDirection::from_exit(exit_endpoint) => {
            tracing::warn!(
                "Vehicle {:?} is on rail {:?} with unexpected reserved direction {:?}",
                args.vehicle_entity,
                args.rail,
                inner.direction,
            );
            return None;
        }
        Some(_) => {}
    }

    let vehicle_list =
        conduit.vehicles.inspect_log("rail conduit owning vehicle must have vehicle list")?;
    let index_in_list = vehicle_list
        .iter()
        .position(|v| v == args.vehicle_entity)
        .inspect_log("rail conduit owning vehicle must contain vehicle in list")?;
    let next_stop = match exit_endpoint {
        AlphaOrBeta::Alpha => {
            match index_in_list.checked_sub(1).and_then(|minus_one| vehicle_list.get(minus_one)) {
                Some(value) => NextStop::Vehicle(value),
                None => NextStop::Endpoint(0.0),
            }
        }
        AlphaOrBeta::Beta => match vehicle_list.get(index_in_list + 1) {
            Some(value) => NextStop::Vehicle(value),
            None => NextStop::Endpoint(corridor.corridor.length),
        },
    };

    let (safety_distance, reaction_time) = if let NextStop::Vehicle(next_entity) = next_stop {
        let &Location::Rail { distance_from_alpha: next_displace, .. } =
            params.vehicle_location_query.log_get(next_entity)?
        else {
            tracing::warn!(
                "rail conduit owning vehicle {:?} must have next vehicle {:?} on the same rail",
                args.vehicle_entity,
                next_entity
            );
            return None;
        };
        let padding = def.physical.length * 0.5 + params.config.safety_headroom;
        let distance = match exit_endpoint {
            AlphaOrBeta::Alpha => (args.displace - next_displace) - padding,
            AlphaOrBeta::Beta => (next_displace - args.displace) - padding,
        };
        (distance, params.config.reaction_time.as_secs_f32())
    } else {
        let distance_from_exit = match exit_endpoint {
            AlphaOrBeta::Alpha => args.displace,
            AlphaOrBeta::Beta => corridor.corridor.length - args.displace,
        };

        (distance_from_exit + def.physical.length * 0.5, 0.0)
    };

    let max_speed = max_stoppable_speed(safety_distance, def.motion.max_braking, reaction_time)
        .min(def.motion.max_speed)
        .min(conduit.rail.max_speed);

    Some(propulsion::Desired::Rail {
        speed_from_alpha: match exit_endpoint {
            AlphaOrBeta::Alpha => -max_speed,
            AlphaOrBeta::Beta => max_speed,
        },
    })
}

fn identify_endpoint<Ab: Which>(
    params: &ControlVehicleParams,
    edge: Option<&edge::CorridorEdge<Ab>>,
    building: Entity,
    choose: impl for<'t> Fn(
        (Option<&'t edge::OfBuilding<Alpha>>, Option<&'t edge::OfBuilding<Beta>>),
    ) -> Option<&'t edge::OfBuilding<Ab>>,
) -> Option<AlphaOrBeta> {
    let edge = edge?.edge();
    let of_building = choose(params.edge_query.log_get(edge)?)?;
    if of_building.0 == building { Some(Ab::default().proto()) } else { None }
}

fn max_stoppable_speed(distance: f32, braking: f32, reaction_time: f32) -> f32 {
    // s = v^2 / 2a + v*t
    // => v = (sqrt(2*a*s + t^2) + t) / a

    ((2.0 * braking * distance + reaction_time.powi(2)).sqrt() + reaction_time) / braking
}
