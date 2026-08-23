use std::borrow::Cow;

use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{EntityCommand, Query, SystemParam};
use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::graph::facility::{self, PersistTypes, blueprint};
use crate::graph::{Facility, building};
use crate::persist::{Depend, InputContext, OutputContext, Persistable};
use crate::util::EntityWorldMutExt;
use crate::{WorldObject, fluid, persist, reactor, view};

#[derive(Clone)]
pub struct Persist;

pub struct Deps {
    building:      Depend<building::Persist>,
    facility_type: Depend<PersistTypes>,
    fluid_type:    Depend<fluid::PersistTypes>,
}

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "graph:facility" }

    type Deps = Deps;
    fn depends(&self, depends: &mut impl persist::Depends) -> Self::Deps {
        Deps {
            building:      depends.request(building::Persist),
            facility_type: depends.request(PersistTypes),
            fluid_type:    depends.request(fluid::PersistTypes),
        }
    }

    type OutputParams<'w, 's> = OutputParams<'w, 's>;
    type Output = Vec<Entry>;

    fn output(
        &self,
        deps: &Deps,
        params: &mut OutputParams<'_, '_>,
        ctx: &mut OutputContext,
    ) -> Result<Self::Output, ()> {
        params
            .facility_query
            .iter()
            .map(|data| {
                Ok(Entry {
                    id:               ctx.alloc(self, data.entity),
                    name:             data.named.name.clone(),
                    building:         ctx.get_id(deps.building, data.building.0)?,
                    ty:               ctx.get_id(deps.facility_type, data.ty.0)?,
                    blueprint_params: BlueprintParams::extract(&data, ctx)?,
                    fluid:            data.fluid.map(fluid::persist::StorageEntry::from_component),
                })
            })
            .collect::<Result<_, ()>>()
    }

    type Input = Vec<Entry>;
    type InputError = InputError;

    fn input(
        &self,
        deps: &Deps,
        world: &mut World,
        input: Self::Input,
        ctx: &mut InputContext,
    ) -> Result<(), InputError> {
        for entry in input {
            let mut entity = world.spawn((WorldObject,));
            ctx.record(self, entry.id, entity.id())
                .map_err(|err| InputError::RecordIdError { err })?;
            entity.reborrow_scope(|entity| {
                facility::SpawnCommand {
                    name:             Some(entry.name),
                    building:         ctx
                        .resolve_entity(deps.building, entry.building)
                        .map_err(|err| InputError::UnresolvedBuilding { err })?,
                    ty:               ctx
                        .resolve_entity(deps.facility_type, entry.ty)
                        .map_err(|err| InputError::UnresolvedFacilityType { err })?,
                    blueprint_params: entry.blueprint_params.resolve(deps, ctx)?,
                }
                .apply(entity);
                Ok(())
            })?;

            if let Some(entry_fluid) = &entry.fluid {
                let Some(mut fluid) = entity.log_get_mut::<fluid::Storage>() else {
                    return Err(InputError::FluidStorageNotExpectedInBlueprint);
                };
                entry_fluid.apply_to_component(&mut fluid);
            }
        }
        Ok(())
    }
}

#[derive(SystemParam)]
pub struct OutputParams<'w, 's> {
    facility_query: Query<'w, 's, OutputQueryData>,
}

#[derive(QueryData)]
struct OutputQueryData {
    entity:   Entity,
    facility: &'static Facility,
    named:    &'static view::Named,
    building: &'static facility::OfBuilding,
    ty:       &'static facility::FacilityType,
    reactor:  Option<&'static reactor::Facility>,
    fluid:    Option<&'static fluid::Storage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id:               persist::Id,
    pub name:             String,
    pub building:         persist::Id,
    pub ty:               persist::Id,
    pub blueprint_params: BlueprintParams,
    pub fluid:            Option<fluid::persist::StorageEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintParams {
    pub reactor: Option<ReactorParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactorParams {
    pub fluid_storages: Vec<Option<persist::Id>>,
}

impl BlueprintParams {
    fn extract(data: &OutputQueryDataItem, ctx: &OutputContext) -> Result<Self, ()> {
        Ok(Self {
            reactor: data.reactor.map(|reactor| Self::extract_reactor(reactor, ctx)).transpose()?,
        })
    }

    fn extract_reactor(
        reactor: &reactor::Facility,
        ctx: &OutputContext,
    ) -> Result<ReactorParams, ()> {
        Ok(ReactorParams {
            fluid_storages: reactor
                .ports
                .fluid_storages
                .iter()
                .map(|&entity| entity.map(|entity| ctx.get_id_unchecked(entity)).transpose())
                .collect::<Result<_, ()>>()?,
        })
    }

    fn resolve(&self, deps: &Deps, ctx: &InputContext) -> Result<blueprint::Params, InputError> {
        Ok(blueprint::Params { reactor: self.resolve_reactor(deps, ctx)? })
    }

    fn resolve_reactor(
        &self,
        deps: &Deps,
        ctx: &InputContext,
    ) -> Result<Option<blueprint::ReactorParams>, InputError> {
        let Some(reactor) = &self.reactor else { return Ok(None) };
        Ok(Some(blueprint::ReactorParams {
            fluid_storages: reactor
                .fluid_storages
                .iter()
                .enumerate()
                .map(|(index, &id)| {
                    id.map(|id| {
                        ctx.resolve_entity_unchecked(id)
                            .map_err(|err| InputError::UnresolvedFluidStorage { err, index })
                    })
                    .transpose()
                })
                .collect::<Result<_, InputError>>()?,
        }))
    }
}

#[derive(Debug, Snafu)]
pub enum InputError {
    #[snafu(display("Register new ID: {err}"))]
    RecordIdError { err: persist::IdError },
    #[snafu(display("Unresolved building: {err}"))]
    UnresolvedBuilding { err: persist::IdError },
    #[snafu(display("Unresolved facility type: {err}"))]
    UnresolvedFacilityType { err: persist::IdError },
    #[snafu(display("Unresolved fluid storage at index {index}: {err}"))]
    UnresolvedFluidStorage { err: persist::IdError, index: usize },
    #[snafu(display(
        "Blueprint for this facility type did not declare a blueprint type, but the facility \
         declared a fluid storage"
    ))]
    FluidStorageNotExpectedInBlueprint,
}
