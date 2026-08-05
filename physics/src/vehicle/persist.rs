use std::borrow::Cow;

use bevy::ecs::entity::Entity;
use bevy::ecs::query::{QueryData, With};
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::system::{EntityCommand, Query, SystemParam};
use bevy::ecs::world::World;
use bevy::math::Vec3;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::graph::{building, conduit, corridor};
use crate::persist::{Depend, InputContext, OutputContext, Persistable};
use crate::util::QueryExt;
use crate::vehicle::{
    self, CompartmentList, CompartmentOf, CompartmentPassengerList, Location, LocationBuilding,
    LocationRail, SpawnCommand, TypeId, Vehicle, propulsion,
};
use crate::{WorldObject, fluid, persist, resident, view};

#[derive(Clone)]
pub struct Persist;

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "vehicle" }

    fn depends(&self) -> impl IntoIterator<Item = Depend> {
        [
            Depend::new(fluid::PersistTypes),
            Depend::new(resident::PersistAttrTypes),
            Depend::new(building::Persist),
            Depend::new(corridor::Persist),
            Depend::new(conduit::Persist),
            Depend::new(vehicle::PersistTypes),
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
            .vehicle_query
            .iter()
            .map(|data| {
                Ok(Entry {
                    id:           ctx.alloc(data.entity),
                    ty:           data.vehicle.ty.0,
                    name:         data.named.name.clone(),
                    location:     match *data.location {
                        Location::Building(location) => EntryLocation::Building {
                            building:     ctx.get_id(location.building)?,
                            interior_pos: location.interior_pos,
                            speed:        location.speed,
                        },
                        Location::Rail(location) => EntryLocation::Rail {
                            conduit:             ctx.get_id(location.rail)?,
                            distance_from_alpha: location.distance_from_alpha,
                            speed_from_alpha:    location.speed_from_alpha,
                        },
                    },
                    compartments: data
                        .compartments
                        .iter()
                        .flat_map(|list| list.iter())
                        .map(|cpmt_entity| {
                            let cpmt = params.compartment_query.log_get(cpmt_entity).ok_or(())?;
                            Ok(CompartmentEntry {
                                fluid: fluid::persist::StorageEntry::from_component(cpmt.fluid),
                            })
                        })
                        .collect::<Result<_, ()>>()?,
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
                SpawnCommand {
                    ty:       TypeId(entry.ty),
                    location: match entry.location {
                        EntryLocation::Building { building, interior_pos, speed } => {
                            Location::Building(LocationBuilding {
                                building: ctx
                                    .resolve_entity(building)
                                    .map_err(|err| InputError::UnresolvedBuilding { err })?,
                                interior_pos,
                                speed,
                            })
                        }
                        EntryLocation::Rail { conduit, distance_from_alpha, speed_from_alpha } => {
                            Location::Rail(LocationRail {
                                rail: ctx
                                    .resolve_entity(conduit)
                                    .map_err(|err| InputError::UnresolvedConduit { err })?,
                                distance_from_alpha,
                                speed_from_alpha,
                            })
                        }
                    },
                    name:     Some(entry.name),
                }
                .apply(entity);

                Ok(())
            })?;

            let num_cpmts = entry.compartments.len();
            for (n, cpmt) in entry.compartments.into_iter().enumerate() {
                let cpmt_entity =
                    entity.get::<CompartmentList>().and_then(|list| list.nth(n)).ok_or(
                        InputError::MismatchCompartmentCount { ty: entry.ty, count: num_cpmts },
                    )?;
                entity.world_scope(|world| {
                    let mut cpmt_entity = world.entity_mut(cpmt_entity);

                    let mut storage = cpmt_entity
                        .get_mut::<fluid::Storage>()
                        .expect("Compartment should have fluid storage");
                    cpmt.fluid.apply_to_component(&mut storage);

                    Ok(())
                })?;
            }
        }
        Ok(())
    }
}

#[derive(SystemParam)]
pub struct OutputParams<'w, 's> {
    vehicle_query:     Query<'w, 's, OutputQueryData>,
    compartment_query: Query<'w, 's, CompartmentQueryData, With<CompartmentOf>>,
}

#[derive(QueryData)]
struct OutputQueryData {
    entity:            Entity,
    vehicle:           &'static Vehicle,
    location:          &'static Location,
    propulsion_status: &'static propulsion::Status,
    named:             &'static view::Named,
    compartments:      Option<&'static CompartmentList>,
}

#[derive(QueryData)]
struct CompartmentQueryData {
    fluid:      &'static fluid::Storage,
    passengers: Option<&'static CompartmentPassengerList>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id:           persist::Id,
    pub ty:           u32,
    pub name:         String,
    pub location:     EntryLocation,
    pub compartments: Vec<CompartmentEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompartmentEntry {
    pub fluid: fluid::persist::StorageEntry,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryLocation {
    Building { building: persist::Id, interior_pos: Vec3, speed: Vec3 },
    Rail { conduit: persist::Id, distance_from_alpha: f32, speed_from_alpha: f32 },
}

#[derive(Debug, Snafu)]
pub enum InputError {
    #[snafu(display("Unresolved building: {err}"))]
    UnresolvedBuilding { err: persist::UnresolvedIdError },
    #[snafu(display("Unresolved conduit: {err}"))]
    UnresolvedConduit { err: persist::UnresolvedIdError },
    #[snafu(display("Mismatch compartment count for vehicle type {ty}, got {count}"))]
    MismatchCompartmentCount { ty: u32, count: usize },
}
