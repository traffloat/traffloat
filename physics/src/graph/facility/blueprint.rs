//! Blueprints define how new facilities can be constructed.
//!
//! Unlike the instance component (e.g. [`reactor::Facility`]) created on the actual facilities,
//! blueprints only specify restrictions on the facility type,
//! such as whether a reactor port can be connected to ambient storage, an adjacent corridor, etc,
//! while the actual instance component would point to the actual storage selected.

use bevy::ecs::entity::Entity;
use bevy::ecs::system::EntityCommand;
use bevy::ecs::world::EntityWorldMut;
use bevy::reflect::Reflect;
use enum_map::EnumMap;
use serde::{Deserialize, Serialize};

use crate::{fluid, reactor, resident};

// This type is currently a struct instead of an enum
// in case a combination of multiple functional components is needed in the future.
// For now, it does not make sense to have e.g.
// both fluid storage and reactor in the same blueprint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Blueprint {
    pub fluid_storage:     Option<FluidStorage>,
    pub reactor:           Option<Reactor>,
    pub interaction_slots: Vec<InteractionSlot>,
}

impl Blueprint {
    #[must_use = "execution is deferred to get an EntityWorldMut"]
    pub fn populate(&self, params: &Params) -> impl FnOnce(&mut EntityWorldMut) + use<> {
        let exec_fluid = self.insert_fluid();
        let exec_reactor = self.insert_reactor(params);
        let exec_interaction_slots = self.insert_interaction_slots();

        move |entity| {
            if let Some(f) = exec_fluid {
                f(entity);
            }
            if let Some(f) = exec_reactor {
                f(entity);
            }
            if let Some(f) = exec_interaction_slots {
                f(entity);
            }
        }
    }

    fn insert_fluid(&self) -> Option<impl FnOnce(&mut EntityWorldMut) + use<>> {
        let def = self.fluid_storage.as_ref()?;
        let command = fluid::AddStorageCommand {
            volume:         def.volume,
            optical_length: def.optical_length,
        };
        Some(move |entity: &mut EntityWorldMut| {
            entity.reborrow_scope(|entity| command.apply(entity));
        })
    }

    fn insert_reactor(&self, params: &Params) -> Option<impl FnOnce(&mut EntityWorldMut) + use<>> {
        let def = self.reactor.as_ref()?;
        let params = try_log!(params.reactor.as_ref(), expect "reactor blueprint expects reactor params" or return None);
        let reactor = reactor::Facility {
            id:             def.ty,
            efficiency_cap: 1.0,
            ports:          reactor::Ports { fluid_storages: params.fluid_storages.clone() },
        };
        Some(move |entity: &mut EntityWorldMut| {
            entity.insert(reactor);
        })
    }

    fn insert_interaction_slots(&self) -> Option<impl FnOnce(&mut EntityWorldMut) + use<>> {
        if self.interaction_slots.is_empty() {
            return None;
        }
        let slots = resident::InteractionSlots {
            slots: self
                .interaction_slots
                .iter()
                .map(|slot| resident::InteractionSlot {
                    name:     slot.name.clone(),
                    capacity: slot.capacity,
                    usage:    0,
                })
                .collect(),
        };
        Some(move |entity: &mut EntityWorldMut| {
            entity.insert(slots);
        })
    }
}

#[derive(Debug, Default)]
pub struct Params {
    pub reactor: Option<ReactorParams>,
}

/// Constrains changes to a [`crate::fluid::Storage`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidStorage {
    /// Volume capacity of the fluid storage.
    ///
    /// This should be less than or equal to the volume of the facility.
    pub volume: f32,

    /// Length of the fluid storage in terms of light absorption
    pub optical_length: f32,
}

/// Constrains changes to a [`reactor::Facility`].
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Reactor {
    pub ty:    reactor::TypeId,
    pub ports: Ports,
}

/// Constrains changes to [`reactor::Ports`].
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Ports {
    pub fluid_storages: Vec<FluidStoragePort>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Reflect)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, enum_map::Enum, Serialize, Deserialize)]
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

#[derive(Debug)]
pub struct ReactorParams {
    pub fluid_storages: Vec<Option<Entity>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionSlot {
    pub name:     String,
    pub capacity: u32,
}
