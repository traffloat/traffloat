use std::borrow::Cow;

use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{EntityCommand, Query, SystemParam};
use bevy::ecs::world::World;
use bevy::math::Vec3;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::graph::{building, corridor, facility};
use crate::persist::{Depend, InputContext, OutputContext, Persistable};
use crate::resident::Resident;
use crate::util::EntityWorldMutExt;
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
            .building_query
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
                                slot_index: interact.slot_index,
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
                                facility: ctx
                                    .resolve_entity(facility)
                                    .map_err(|err| InputError::UnresolvedFacility { err })?,
                                slot_index,
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
    building_query: Query<'w, 's, OutputQueryData>,
}

#[derive(QueryData)]
struct OutputQueryData {
    entity:      Entity,
    resident:    &'static Resident,
    location:    &'static resident::Location,
    attrs:       &'static resident::Attributes,
    named:       &'static view::Named,
    interaction: Option<&'static resident::InteractingWith>,
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
    Facility { facility: persist::Id, slot_index: usize },
}

#[derive(Debug, Snafu)]
pub enum InputError {
    #[snafu(display("Unresolved building: {err}"))]
    UnresolvedBuilding { err: persist::UnresolvedIdError },
    #[snafu(display("Unresolved corridor: {err}"))]
    UnresolvedCorridor { err: persist::UnresolvedIdError },
    #[snafu(display("Unresolved facility: {err}"))]
    UnresolvedFacility { err: persist::UnresolvedIdError },
    #[snafu(display("Resident attributes length mismatch: expected {expected}, got {got}"))]
    AttributesLengthMismatch { expected: usize, got: usize },
}
