use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::EntityCommand;
use bevy::ecs::world::EntityWorldMut;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};
use traffloat_proto::proto::AlphaOrBeta;

use crate::vehicle::def::GaugeSize;

pub mod persist;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Rail>();
        app.register_type::<Reservation>();
    }
}

#[derive(Debug, Clone, Component, Serialize, Deserialize, Reflect)]
pub struct Rail {
    pub gauge_size:  GaugeSize,
    pub electrified: bool,
    pub max_speed:   f32,
    // TODO further restriction labels
}

/// When `inner` is `Some`, the rail is reserved in a given direction.
/// Vehicles may only enter from the reserved direction,
/// and inertial entry is only allowed when they can brake before the clearance marker.
///
/// Reservation is greedy, so it may result in lock starvation.
/// The existence of reservation is to avoid collision, not for traffic control.
/// A higher-level traffic control system should be applied at the pathfinding layer
/// to avoid starvation scenario from reaching the motion layer.
#[derive(Component, Default, Serialize, Deserialize, Reflect)]
pub struct Reservation {
    pub inner: Option<ReservationInner>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
pub struct ReservationInner {
    /// Direction of the reservation.
    /// Vehicles may only enter the rail in this direction.
    pub direction: ReservedDirection,

    /// The rail is reserved by a vehicle that has not yet entered the rail.
    ///
    /// This field is set to `None` upon entry of the vehicle.
    pub external_vehicle: Option<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum ReservedDirection {
    AlphaToBeta,
    BetaToAlpha,
}

impl ReservedDirection {
    pub fn from_entry(entry: AlphaOrBeta) -> Self {
        match entry {
            AlphaOrBeta::Alpha => ReservedDirection::AlphaToBeta,
            AlphaOrBeta::Beta => ReservedDirection::BetaToAlpha,
        }
    }

    pub fn from_exit(exit: AlphaOrBeta) -> Self {
        match exit {
            AlphaOrBeta::Alpha => ReservedDirection::BetaToAlpha,
            AlphaOrBeta::Beta => ReservedDirection::AlphaToBeta,
        }
    }

    pub fn entry(self) -> AlphaOrBeta {
        match self {
            ReservedDirection::AlphaToBeta => AlphaOrBeta::Alpha,
            ReservedDirection::BetaToAlpha => AlphaOrBeta::Beta,
        }
    }

    pub fn exit(self) -> AlphaOrBeta {
        match self {
            ReservedDirection::AlphaToBeta => AlphaOrBeta::Beta,
            ReservedDirection::BetaToAlpha => AlphaOrBeta::Alpha,
        }
    }
}

pub struct SpawnCommand {
    pub rail:         Rail,
    pub reserved_dir: Option<ReservedDirection>,
}

impl EntityCommand for SpawnCommand {
    type Out = ();

    fn apply(self, mut entity: EntityWorldMut) {
        entity.insert((
            self.rail,
            Reservation {
                inner: self
                    .reserved_dir
                    .map(|direction| ReservationInner { direction, external_vehicle: None }),
            },
        ));
    }
}
