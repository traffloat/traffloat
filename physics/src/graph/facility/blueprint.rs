//! Blueprints define how new facilities can be constructed.
//!
//! Unlike the instance component (e.g. [`reactor::Facility`]) created on the actual facilities,
//! blueprints only specify restrictions on the facility type,
//! such as whether a reactor port can be connected to ambient storage, an adjacent corridor, etc,
//! while the actual instance component would point to the actual storage selected.

use bevy::ecs::entity::Entity;
use bevy::reflect::Reflect;
use enum_map::EnumMap;

use crate::reactor;

// This type is currently a struct instead of an enum
// in case a combination of multiple functional components is needed in the future.
// For now, it does not make sense to have e.g.
// both fluid storage and reactor in the same blueprint.
#[derive(Debug, Clone, Default)]
pub struct Blueprint {
    pub fluid_storage: Option<FluidStorage>,
    pub reactor:       Option<Reactor>,
}

/// Constrains changes to a [`crate::fluid::Storage`].
#[derive(Debug, Clone)]
pub struct FluidStorage {
    /// Volume capacity of the fluid storage.
    ///
    /// This should be less than or equal to the volume of the facility.
    pub volume: f32,

    /// Length of the fluid storage in terms of light absorption
    pub optical_length: f32,
}

/// Constrains changes to a [`reactor::Facility`].
#[derive(Debug, Clone, Reflect)]
pub struct Reactor {
    pub ty:    reactor::TypeId,
    pub ports: Ports,
}

/// Constrains changes to [`reactor::Ports`].
#[derive(Debug, Clone, Reflect)]
pub struct Ports {
    pub fluid_storages: Vec<FluidStoragePort>,
}

#[derive(Debug, Clone, Default, Reflect)]
pub struct FluidStoragePort {
    /// Which port types are compatible.
    #[reflect(ignore, default)]
    pub compat: EnumMap<FluidStoragePortType, bool>,
}

impl FluidStoragePort {
    #[must_use]
    pub fn with(mut self, ty: FluidStoragePortType) -> Self {
        self.compat[ty] = true;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, enum_map::Enum)]
pub enum FluidStoragePortType {
    /// This port can be unconnected. Only makes sense for inputs/catalysts.
    Nil,
    /// This port can be connected to the ambient fluid of the building.
    Ambient,
    /// This port can be connected to a fluid storage in the same building.
    OtherFacility,
    /// This port can be connected to the fluid storage of a fluid conduit in an adjacent corridor.
    AdjacentPipe,
}

#[derive(Debug, Default)]
pub struct Params {
    pub reactor: Option<ReactorParams>,
}

#[derive(Debug)]
pub struct ReactorParams {
    pub fluid_storages: Vec<Option<Entity>>,
}
