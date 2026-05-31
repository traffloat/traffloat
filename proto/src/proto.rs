use std::num::NonZeroU32;

use bevy::color::LinearRgba;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

pub type Vector = bevy::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub struct Id(pub NonZeroU32);

/// Represents a linear RGBA color.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
pub struct Color(pub [f32; 4]);

impl From<bevy::color::Color> for Color {
    fn from(value: bevy::color::Color) -> Self { value.to_linear().into() }
}

impl From<Color> for bevy::color::Color {
    fn from(value: Color) -> Self { LinearRgba::from(value).into() }
}

impl From<LinearRgba> for Color {
    fn from(value: LinearRgba) -> Self { Color([value.red, value.green, value.blue, value.alpha]) }
}

impl From<Color> for LinearRgba {
    fn from(Color([red, green, blue, alpha]): Color) -> Self {
        LinearRgba { red, green, blue, alpha }
    }
}

/// Messages from the world to a specific viewer.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum Update {
    NewBuilding(NewBuilding),
    UpdateBuilding(UpdateBuilding),
    UpdateBuildingFull(UpdateBuildingFull),
    NewCorridor(NewCorridor),
    UpdateCorridor(UpdateCorridor),
    UpdateCorridorFull(UpdateCorridorFull),
    SetCorridorEndpoint(SetCorridorEndpoint),
    NewFacility(NewFacility),
    SetFacilityTaint(SetFacilityTaint),
    RemoveViewable(RemoveViewable),
    SetFluidTypes(SetFluidTypes),
}

/// Subscribed to a new building.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct NewBuilding {
    pub id:             Id,
    pub name:           String,
    pub position:       Vector,
    pub radius:         f32,
    pub wall_thickness: f32,
}

/// Updated information about a building.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct UpdateBuilding {
    pub id:    Id,
    pub color: Color,
}

/// Updated full information about a building.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct UpdateBuildingFull {
    pub id:    Id,
    pub color: Color,

    pub ambient_fluid: FluidStorageFull,
}

/// Subscribed to a new corridor.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct NewCorridor {
    pub id:             Id,
    pub name:           String,
    pub alpha_position: Vector,
    pub beta_position:  Vector,
    pub radius:         f32,
    pub wall_thickness: f32,
}

/// Updated information about a building.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct UpdateCorridor {
    pub id:    Id,
    pub color: Color,
}

/// Updated full information about a corridor.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct UpdateCorridorFull {
    pub id:    Id,
    pub color: Color,

    pub ambient_fluid: FluidStorageFull,
}

/// Set or unset the endpoint building of a corridor.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct SetCorridorEndpoint {
    pub corridor: Id,
    pub which:    AlphaOrBeta,
    pub value:    Option<CorridorEndpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct CorridorEndpoint {
    pub building: Id,
    pub open:     bool,
}

/// Subscribed to a new facility in an existing building.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct NewFacility {
    pub id:       Id,
    pub building: Id,
    pub name:     String,
    pub volume:   f32,
    pub display:  FacilityDisplay,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct FacilityDisplay {
    pub sprite_id: String,
    pub taint:     Color,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct SetFacilityTaint {
    pub id:    Id,
    pub taint: Color,
}

/// Unsubscribed from a viewable.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct RemoveViewable {
    pub id: Id,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct SetFluidTypes {
    pub types: Vec<FluidType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct FluidType {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct FluidStorageFull {
    pub volume:      f32,
    pub pressure:    f32,
    pub temperature: f32,
    pub types:       Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum AlphaOrBeta {
    Alpha,
    Beta,
}

/// Approved messages from a specific viewer to the world.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum Request {
    Handshake,
}
