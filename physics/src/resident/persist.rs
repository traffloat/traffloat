use std::borrow::Cow;

use bevy::ecs::entity::Entity;
use bevy::ecs::query::{QueryData, With};
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::system::{EntityCommand, Query, SystemParam};
use bevy::ecs::world::World;
use bevy::math::Vec3;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::graph::{building, corridor, facility};
use crate::persist::{Depend, InputContext, OutputContext, Persistable};
use crate::resident::Resident;
use crate::util::{EntityWorldMutExt, QueryExt};
use crate::vehicle::{self, Vehicle};
use crate::{WorldObject, persist, resident, view};

#[derive(Clone)]
pub struct Persist;

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "resident" }

    fn depends(&self) -> impl IntoIterator<Item = Depend> {
        [
            Depend::new(resident::PersistAttrTypes),
            Depend::new(building::Persist),
            Depend::new(corridor::Persist),
            Depend::new(facility::Persist),
            Depend::new(vehicle::Persist),
        ]
    }

    type OutputParams<'w, 's> = OutputParams<'w, 's>;
    type Output = Vec<Entry>;

    fn output(
        &self,
        params: &mut OutputParams<'_, '_>,
        ctx: &mut OutputContext,
    ) -> Result<Self::Output, ()> {
        params
            .resident_query
            .iter()
            .map(|data| {
                Ok(Entry {
                    id:       ctx.alloc(data.entity),
                    name:     data.named.name.clone(),
                    attrs:    data.attrs.values.clone().into(),
                    location: match *data.location {
                        resident::Location::Building { entity, interior_pos } => {
                            EntryLocation::Building { building: ctx.get_id(entity)?, interior_pos }
                        }
                        resident::Location::Corridor { entity, distance_from_alpha } => {
                            EntryLocation::Corridor {
                                corridor: ctx.get_id(entity)?,
                                distance_from_alpha,
                            }
                        }
                        resident::Location::Facility { entity } => {
                            let interact = if let Some(interact) = data.interaction
                                && interact.facility == entity
                            {
                                interact
                            } else {
                                tracing::error!(
                                    "Resident {:?} location is in {entity:?} but interaction is \
                                     {:?}",
                                    data.entity,
                                    data.interaction
                                );
                                return Err(());
                            };
                            EntryLocation::Facility {
                                facility:   ctx.get_id(entity)?,
                                slot_index: u32::try_from(interact.slot_index)
                                    .expect("too many interaction slots"),
                            }
                        }
                        resident::Location::Vehicle { compartment } => {
                            let cpmt_data =
                                params.compartment_query.log_get(compartment).ok_or(())?;
                            let vehicle = ctx.get_id(cpmt_data.vehicle.0)?;
                            let vehicle_data =
                                params.vehicle_query.log_get(cpmt_data.vehicle.0).ok_or(())?;
                            let as_passenger = try_log!(data.passenger, expect "location vehicle implies passenger component" or return Err(()));
                            let cpmt_index = as_passenger.compartment_index;
                            let operator_slot = data.operator.map(|op| {
                                debug_assert_eq!(
                                    op.vehicle, cpmt_data.vehicle.0,
                                    "operator vehicle does not match compartment vehicle"
                                );
                                u32::try_from(op.slot).expect("too many operator slots")
                            });
                            EntryLocation::Vehicle {
                                vehicle,
                                compartment: u32::try_from(cpmt_index)
                                    .expect("too many compartments"),
                                operator_slot,
                            }
                        }
                    },
                })
            })
            .collect()
    }

    type Input = Vec<Entry>;
    type InputError = InputError;

    fn input(
        &self,
        world: &mut World,
        input: Self::Input,
        ctx: &mut InputContext,
    ) -> Result<(), InputError> {
        for entry in input {
            let mut entity = world.spawn((WorldObject,));
            ctx.record(entry.id, entity.id());

            entity.reborrow_scope(|entity| {
                resident::SpawnCommand {
                    name: Some(entry.name),
                    at:   match entry.location {
                        EntryLocation::Building { building, interior_pos } => {
                            resident::SpawnAt::Building {
                                building: ctx
                                    .resolve_entity(building)
                                    .map_err(|err| InputError::UnresolvedBuilding { err })?,
                                interior_pos,
                            }
                        }
                        EntryLocation::Corridor { corridor, distance_from_alpha } => {
                            resident::SpawnAt::Corridor {
                                corridor: ctx
                                    .resolve_entity(corridor)
                                    .map_err(|err| InputError::UnresolvedCorridor { err })?,
                                distance_from_alpha,
                            }
                        }
                        EntryLocation::Facility { facility, slot_index } => {
                            resident::SpawnAt::Facility {
                                facility:   ctx
                                    .resolve_entity(facility)
                                    .map_err(|err| InputError::UnresolvedFacility { err })?,
                                slot_index: usize::try_from(slot_index).expect("usize >= u32"),
                            }
                        }
                        EntryLocation::Vehicle {
                            vehicle,
                            compartment: compartment_index,
                            operator_slot,
                        } => {
                            let vehicle_entity = ctx
                                .resolve_entity(vehicle)
                                .map_err(|err| InputError::UnresolvedVehicle { err })?;
                            let vehicle = entity
                                .world()
                                .get::<Vehicle>(vehicle_entity)
                                .expect("vehicle must have vehicle component");
                            let cpmt_list = entity
                                .world()
                                .get::<vehicle::CompartmentList>(vehicle_entity)
                                .expect("vehicle must have compartment list");
                            let compartment_entity = cpmt_list
                                .nth(usize::try_from(compartment_index).expect("usize >= u32"))
                                .ok_or(InputError::InvalidCompartment {
                                    compartment: compartment_index,
                                })?;
                            let ty = entity.resource::<vehicle::Types>().get(vehicle.ty);
                            if let Some(slot) = operator_slot
                                && usize::try_from(slot).expect("usize >= u32")
                                    >= ty.operator_slots.len()
                            {
                                return Err(InputError::InvalidOperatorSlot {
                                    operator_slot: slot,
                                });
                            }
                            resident::SpawnAt::Vehicle {
                                vehicle:           vehicle_entity,
                                compartment:       compartment_entity,
                                compartment_index: usize::try_from(compartment_index)
                                    .expect("usize >= u32"),
                                operator_slot:     operator_slot
                                    .map(|slot| usize::try_from(slot).expect("usize >= u32")),
                            }
                        }
                    },
                }
                .apply(entity);
                Ok(())
            })?;

            if let Some(mut attrs) = entity.log_get_mut::<resident::Attributes>() {
                if attrs.values.len() != entry.attrs.len() {
                    return Err(InputError::AttributesLengthMismatch {
                        expected: attrs.values.len(),
                        got:      entry.attrs.len(),
                    });
                }
                attrs.values.copy_from_slice(&entry.attrs);
            }
        }
        Ok(())
    }
}

#[derive(SystemParam)]
pub struct OutputParams<'w, 's> {
    resident_query:    Query<'w, 's, OutputQueryData>,
    vehicle_query:     Query<'w, 's, OutputVehicleQueryData, With<Vehicle>>,
    compartment_query: Query<'w, 's, OutputCompartmentQueryData>,
}

#[derive(QueryData)]
struct OutputQueryData {
    entity:      Entity,
    resident:    &'static Resident,
    location:    &'static resident::Location,
    attrs:       &'static resident::Attributes,
    named:       &'static view::Named,
    interaction: Option<&'static resident::InteractingWith>,
    passenger:   Option<&'static vehicle::PassengerOfCompartment>,
    operator:    Option<&'static vehicle::OperatorOf>,
}

#[derive(QueryData)]
struct OutputVehicleQueryData {
    compartments: &'static vehicle::CompartmentList,
    operators:    Option<&'static vehicle::OperatorList>,
}

#[derive(QueryData)]
struct OutputCompartmentQueryData {
    vehicle: &'static vehicle::CompartmentOf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id:       persist::Id,
    pub name:     String,
    pub attrs:    Vec<f32>,
    pub location: EntryLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryLocation {
    Building { building: persist::Id, interior_pos: Vec3 },
    Corridor { corridor: persist::Id, distance_from_alpha: f32 },
    Facility { facility: persist::Id, slot_index: u32 },
    Vehicle { vehicle: persist::Id, compartment: u32, operator_slot: Option<u32> },
}

#[derive(Debug, Snafu)]
pub enum InputError {
    #[snafu(display("Unresolved building: {err}"))]
    UnresolvedBuilding { err: persist::UnresolvedIdError },
    #[snafu(display("Unresolved corridor: {err}"))]
    UnresolvedCorridor { err: persist::UnresolvedIdError },
    #[snafu(display("Unresolved facility: {err}"))]
    UnresolvedFacility { err: persist::UnresolvedIdError },
    #[snafu(display("Unresolved vehicle: {err}"))]
    UnresolvedVehicle { err: persist::UnresolvedIdError },
    #[snafu(display("Invalid compartment index for this vehicle type: {compartment}"))]
    InvalidCompartment { compartment: u32 },
    #[snafu(display("Invalid operator slot for this vehicle type: {operator_slot}"))]
    InvalidOperatorSlot { operator_slot: u32 },
    #[snafu(display("Resident attributes length mismatch: expected {expected}, got {got}"))]
    AttributesLengthMismatch { expected: usize, got: usize },
}
