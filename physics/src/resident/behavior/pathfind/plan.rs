//! Plan commutes for residents.
//!
//! # Goal
//! The design of this system considers the following scenarios.
//!
//! ## Commute targets
//! - A specific building to interact with a facility there, e.g. operator, user.
//! - A specific vehicle compartment as part of work, e.g. driver, secondary operator.
//! - A specific corridor endpoint for construction.
//! - A specific corridor location for incident response, e.g. firefighting, cleaning.
//!
//! ## Commute methods
//! Use of vehicles speeds up commutes but is subject to availability and restrictions.
//!
//! There are two primary choices of vehicle usage, subject to different restrictions:
//! - Private driving:
//!   - Normal commute should only use vehicles intended for private driving,
//!     e.g. they should avoid special vehicles or mass transit vehicles.
//!   - Vehicles currently in use should not be considered for private driving.
//!   - Residents can only use vehicles which their attributes support,
//!     e.g. special driving skills.
//! - Mass transit (initially not implemented):
//!   - Subject to time constraint waiting for the vehicle to arrive.
//!     Requires timetabling.
//!
//! Furthermore, specific tasks require that the resident
//! arrive at the commute target in a specific vehicle/vehicle type,
//! e.g. a firefighter shall arrive in a fire brigade vehicle.
//!
//! # Algorithm
//! Initially we just use simple unidirectional Dijkstra since
//! any premature optimization is likely to be obsolete as new features are implemented.
//!
//! Potential optimizations include:
//! - Caching node-to-node path lengths
//! - Finding the nearest vehicles and perform A* from the vehicle location to the target.

use bevy::app::{App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::{Changed, QueryData};
use bevy::ecs::system::Query;
use bevy::math::{Vec2, Vec3};
use bevy::reflect::Reflect;

use crate::{resident, vehicle};

mod pathfinder;
use pathfinder::Pathfinder;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Input>();
        app.register_type::<Output>();
    }
}

#[derive(Component, Reflect, Default)]
pub enum Input {
    /// Do not try to control movement.
    #[default]
    Undetermined,
    Pathfind(InputPathfind),
}

#[derive(Reflect)]
pub struct InputPathfind {
    /// The target location.
    pub target:         Target,
    /// Restrictions on transportation methods used.
    pub vehicle_policy: VehiclePolicy,
}

#[derive(Reflect)]
pub enum Target {
    /// Pursue a position in a building.
    Building(TargetBuilding),
    /// Pursue a position in a corridor.
    Corridor(TargetCorridor),
    /// Pursue entry into a vehicle compartment.
    Vehicle(TargetVehicle),
}

#[derive(Reflect)]
pub struct TargetBuilding {
    pub building:     Entity,
    /// If unspecified, target is fulfilled upon entering anywhere in the building.
    pub interior_pos: Option<Vec3>,
}

#[derive(Reflect)]
pub struct TargetCorridor {
    pub corridor:            Entity,
    /// If unspecified, target is fulfilled upon entering anywhere in the corridor longitudinally.
    /// Otherwise, pursues the cross section at the specified distance from the alpha endpoint.
    pub distance_from_alpha: Option<f32>,
    pub cross_section:       TargetCorridorCrossSection,
}

#[derive(Reflect)]
pub enum TargetCorridorCrossSection {
    /// Pursues the nearest point on the cross section.
    Unspecified,
    /// Pursues a specific position on the cross section at `distance_from_alpha`.
    Position(Vec2),
    /// Enters a specific rail, stopping at `distance_from_alpha`.
    Rail(Entity),
}

#[derive(Reflect)]
pub struct TargetVehicle {
    pub compartment: Entity,
}

#[derive(Reflect)]
pub enum VehiclePolicy {
    /// Deny any vehicle usage.
    WalkingOnly,
    /// Allow any vehicle usage.
    AnyVehicle,
    /// Allow only a specific vehicle.
    SpecificVehicle(Entity),
    /// Allow only a specific vehicle type.
    SpecificVehicleType(vehicle::TypeId),
}

#[derive(Component, Reflect)]
pub struct Output {
    plan: Option<Plan>,
}

#[derive(Reflect)]
struct Plan {}

fn update_system(mut resident_query: Query<UpdateResidentData, Changed<Input>>) {
    resident_query.par_iter_mut().for_each(|mut data| update(&mut data));
}

fn update(resident: &mut UpdateResidentDataItem) {
    let Input::Pathfind(input) = resident.input else { return };
    let mut pathfinder = Pathfinder::default();
    match resident.location {
        resident::Location::Building { entity, interior_pos } => {}
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct UpdateResidentData {
    location: &'static resident::Location,
    input:    &'static Input,
    result:   &'static mut Output,
}
