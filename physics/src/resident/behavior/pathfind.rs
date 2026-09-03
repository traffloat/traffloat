use bevy::app::{App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::math::{Vec2, Vec3};
use bevy::reflect::Reflect;

use crate::vehicle;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) { app.register_type::<CommuteInput>(); }
}

#[derive(Component, Reflect, Default)]
pub enum CommuteInput {
    /// Do not try to control movement.
    #[default]
    Undetermined,
    Pathfind(CommuteInputPathfind),
}

#[derive(Reflect)]
pub struct CommuteInputPathfind {
    /// The target location.
    pub target:         CommuteTarget,
    /// Restrictions on transportation methods used.
    pub vehicle_policy: VehiclePolicy,
}

#[derive(Reflect)]
pub enum CommuteTarget {
    /// Pursue a position in a building.
    Building(CommuteTargetBuilding),
    /// Pursue a position in a corridor.
    Corridor(CommuteTargetCorridor),
    /// Pursue entry into a vehicle compartment.
    Vehicle(CommuteTargetVehicle),
}

#[derive(Reflect)]
pub struct CommuteTargetBuilding {
    pub building:     Entity,
    /// If unspecified, target is fulfilled upon entering anywhere in the building.
    pub interior_pos: Option<Vec3>,
}

#[derive(Reflect)]
pub struct CommuteTargetCorridor {
    pub corridor:            Entity,
    /// If unspecified, target is fulfilled upon entering anywhere in the corridor longitudinally.
    /// Otherwise, pursues the cross section at the specified distance from the alpha endpoint.
    pub distance_from_alpha: Option<f32>,
    pub cross_section:       CommuteTargetCorridorCrossSection,
}

#[derive(Reflect)]
pub enum CommuteTargetCorridorCrossSection {
    /// Pursues the nearest point on the cross section.
    Unspecified,
    /// Pursues a specific position on the cross section at `distance_from_alpha`.
    Position(Vec2),
    /// Enters a specific rail, stopping at `distance_from_alpha`.
    Rail(Entity),
}

#[derive(Reflect)]
pub struct CommuteTargetVehicle {
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
