use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

pub type Vector = bevy::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Id(pub NonZeroU32);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color(pub [f32; 4]);

/// Messages from the world to a specific viewer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Update {
    NewBuilding(NewBuilding),
    UpdateBuilding(UpdateBuilding),
    UpdateBuildingFull(UpdateBuildingFull),
    NewCorridor(NewCorridor),
    UpdateCorridor(UpdateCorridor),
    UpdateCorridorFull(UpdateCorridorFull),
    SetCorridorEndpoint(SetCorridorEndpoint),
    RemoveViewable(RemoveViewable),
    SetFluidTypes(SetFluidTypes),
}

/// Subscribed to a new building.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBuilding {
    pub id:             Id,
    pub name:           String,
    pub position:       Vector,
    pub radius:         f32,
    pub wall_thickness: f32,
}

/// Updated information about a building.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBuilding {
    pub id:    Id,
    pub color: Color,
}

/// Updated full information about a building.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBuildingFull {
    pub id:    Id,
    pub color: Color,

    pub ambient_fluid: FluidStorageFull,
}

/// Subscribed to a new corridor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCorridor {
    pub id:             Id,
    pub name:           String,
    pub alpha_position: Vector,
    pub beta_position:  Vector,
    pub radius:         f32,
    pub wall_thickness: f32,
}

/// Updated information about a building.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCorridor {
    pub id:    Id,
    pub color: Color,
}

/// Updated full information about a corridor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCorridorFull {
    pub id:    Id,
    pub color: Color,

    pub ambient_fluid: FluidStorageFull,
}

/// Set or unset the endpoint building of a corridor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCorridorEndpoint {
    pub corridor: Id,
    pub which:    AlphaOrBeta,
    pub value:    Option<CorridorEndpoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorridorEndpoint {
    pub building: Id,
    pub open:     bool,
}

/// Unsubscribed from a viewable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveViewable {
    pub id: Id,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetFluidTypes {
    pub types: Vec<FluidType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidType {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidStorageFull {
    pub volume:      f32,
    pub pressure:    f32,
    pub temperature: f32,
    pub types:       Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlphaOrBeta {
    Alpha,
    Beta,
}

/// Approved messages from a specific viewer to the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Handshake,
}
