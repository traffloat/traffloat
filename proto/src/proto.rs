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
    RemoveViewable(RemoveViewable),
}

/// Subscribed to a new building.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBuilding {
    pub id:             Id,
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

    pub ambient: FluidStorageFull,
}

/// Subscribed to a new corridor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCorridor {
    pub id:             Id,
    pub alpha_position: Vector,
    pub beta_position:  Vector,
    pub radius:         f32,
    pub color:          Color,
}

/// Updated information about a building.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCorridor {
    pub id:    Id,
    pub color: Color,
}

/// Unsubscribed from a viewable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveViewable {
    pub id: Id,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidStorageFull {
    pub pressure:    f32,
    pub temperature: f32,
    pub fluids:      Vec<f32>,
}

/// Approved messages from a specific viewer to the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Handshake,
}
