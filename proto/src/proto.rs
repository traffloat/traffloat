use std::num::NonZeroU32;

use bevy::color::LinearRgba;
use bevy::math::Vec3;
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

/// Messages from the world to a specific viewer.
#[derive(
    Debug, Clone, Serialize, Deserialize, Reflect, strum::IntoStaticStr, derive_more::From,
)]
pub enum Update {
    SetFluidTypes(SetFluidTypes),
    SetResidentAttrTypes(SetResidentAttrTypes),
    NewBuilding(NewBuilding),
    UpdateBuilding(UpdateBuilding),
    UpdateBuildingFull(UpdateBuildingFull),
    UpdateBuildingFluidConnections(UpdateBuildingFluidConnections),
    NewCorridor(NewCorridor),
    UpdateCorridor(UpdateCorridor),
    UpdateCorridorFull(UpdateCorridorFull),
    UpdateCorridorEndpoint(UpdateCorridorEndpoint),
    NewFacility(NewFacility),
    UpdateFacilityTaint(UpdateFacilityTaint),
    UpdateFacilityFluid(UpdateFacilityFluid),
    NewConduit(NewConduit),
    UpdateFluidConduit(UpdateFluidConduit),
    UpdateFluidConduitFull(UpdateFluidConduitFull),
    NewResident(NewResident),
    UpdateResidentLocation(UpdateResidentLocation),
    UpdateResidentAttributesFull(UpdateResidentAttributesFull),
    UpdateResidentAttributesPartial(UpdateResidentAttributesPartial),
    RemoveViewable(RemoveViewable),
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
pub struct SetResidentAttrTypes {
    pub types: Vec<ResidentAttrType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ResidentAttrType {
    pub name:       String,
    pub subscribed: bool,
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
pub struct UpdateCorridorEndpoint {
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

/// Display information about a facility.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct FacilityDisplay {
    /// Asset path.
    ///
    /// Currently loads from `assets/sprites/{sprite_id}.png` directly.
    /// May be extended to support dynamically loaded assets in the future.
    pub sprite_id: String,
    /// `Some` when the facility is a fluid storage, represents the fluid color.
    pub taint:     Option<Color>,
}

/// Sets the taint color of a facility.
///
/// The facility must have been previously created with [`FacilityDisplay::taint`] set to `Some`.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct UpdateFacilityTaint {
    pub id:    Id,
    pub taint: Color,
}

/// Updated fluid information of a facility with a fluid storage.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct UpdateFacilityFluid {
    pub id:    Id,
    pub fluid: FluidStorageFull,
}

/// Sets the fluid connections within a building.
///
/// This does not include building-corridor edges.
/// Building-corridor connections must be either open or closed instead of adjustable area,
/// and are set with [`UpdateCorridorEndpoint`] instead of this message.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct UpdateBuildingFluidConnections {
    pub id:          Id,
    pub connections: Vec<BuildingFluidConnection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BuildingFluidConnection {
    pub current_area: f32,
    pub max_area:     f32,
    pub pair:         BuildingFluidConnectionPair,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum BuildingFluidConnectionPair {
    FacilityFacility(Id, Id),
    FacilityBuilding { facility: Id, building: Id },
    FacilityPipe { facility: Id, pipe: Id },
}

/// Subscribed to a new conduit in an existing corridor.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct NewConduit {
    pub id:       Id,
    pub name:     String,
    pub corridor: Id,
    pub radius:   f32,
    pub ty:       ConduitType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Reflect)]
pub enum ConduitType {
    FluidPipe,
}

/// Updated information about a fluid pipe.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct UpdateFluidConduit {
    pub id:    Id,
    pub color: Color,
}

/// Updated full information about a fluid pipe.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct UpdateFluidConduitFull {
    pub id:    Id,
    pub color: Color,

    pub fluid: FluidStorageFull,
}

/// Subscribed to a new resident.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct NewResident {
    pub id:       Id,
    pub name:     String,
    pub location: ResidentLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct UpdateResidentLocation {
    pub id:       Id,
    pub location: ResidentLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct UpdateResidentAttributesFull {
    pub id:    Id,
    pub attrs: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct UpdateResidentAttributesPartial {
    pub id:    Id,
    pub attrs: Vec<(u32, f32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum ResidentLocation {
    Building { building: Id, interior_pos: Vec3, speed: Vec3 },
    Corridor { corridor: Id, linear_pos: f32, speed: f32 },
    Facility { facility: Id },
}

/// Unsubscribed from a viewable.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct RemoveViewable {
    pub id: Id,
}

/// Approved messages from a specific viewer to the world.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum Request {
    Handshake,
}
