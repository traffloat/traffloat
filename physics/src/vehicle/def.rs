use bevy::reflect::Reflect;
use enum_map::EnumMap;
use serde::{Deserialize, Serialize};

use crate::vehicle::Propulsion;

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Def {
    /// Name of the vehicle type.
    pub name:           String,
    /// Physical properties.
    pub physical:       Physical,
    /// Motion properties.
    pub motion:         Motion,
    /// Compartments in a vehicle.
    pub compartments:   Vec<Compartment>,
    /// Operator slots for the vehicle.
    ///
    /// Note that operators are just passengers with special roles.
    /// A driver resident would take *both* operator slot and compartment passenger slot.
    pub operator_slots: Vec<OperatorSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Physical {
    /// Mass of the vehicle, used for F=ma calculation.
    pub mass:   f32,
    /// Exterior volume of the vehicle, affects ambient fluid volume calculation.
    pub volume: f32,
    /// Length of the vehicle, used for collision detection.
    pub length: f32,
    /// Gauge size of the vehicle, used for rail compatibility.
    pub gauge:  GaugeSize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Motion {
    /// Propulsion source of the vehicle, defined as a reaction.
    ///
    /// This defines the [reaction](crate::reaction) for propulsion
    /// and controls the force that propels the vehicle.
    pub propulsion:       Propulsion,
    /// Maximum speed of the vehicle.
    pub max_speed:        f32,
    /// Maximum braking force of the vehicle.
    ///
    /// This value is always positive.
    pub max_braking:      f32,
    /// Drag coefficient of the vehicle.
    ///
    /// Drag force is determined by `drag_coefficient * pressure * speed^2`.
    pub drag_coefficient: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Compartment {
    pub name:                  String,
    /// Fluid volume of the compartment.
    pub volume:                f32,
    /// Maximum number of passengers that can enter the compartment.
    pub passenger_slots:       u32,
    /// Area of the compartment's connection to the ambient fluid.
    pub vent_area:             f32,
    /// Reciprocal resistance of the compartment's connection to the ambient fluid.
    pub vent_resistance_recip: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct OperatorSlot {
    pub name:  String,
    #[reflect(ignore, default)]
    pub roles: EnumMap<OperatorRole, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, enum_map::Enum)]
pub enum OperatorRole {
    /// Required for the vehicle to move.
    Driver,
    /// Required for the vehicle to load and unload cargo.
    CargoLoader,
}

/// Gauge size represented as a fraction.
///
/// This representation is used instead of a single float value
/// because gauge size needs to be matched exactly.
///
/// The fraction may be converted into `f32` for area calculation
/// when determining whether it fits into a corridor,
/// but otherwise the type [`GaugeSize`] alone is not an ordered type.
/// Only exact equality of both integers are treated as equivalent,
/// e.g. `GaugeSize(3, 4)` and `GaugeSize(6, 8)` are not equivalent.
/// This is subject to ruleset creators to ensure that
/// they use a consistent convention for gauge size representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub struct GaugeSize(pub u16, pub u16);
