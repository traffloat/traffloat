use std::borrow::Cow;

use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{EntityCommand, Query, SystemParam};
use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use traffloat_util::EntityWorldMutExt;

use crate::graph::{Connection, building, conduit, connection, facility};
use crate::persist::{Depend, InputContext, OutputContext, Persistable};
use crate::{WorldObject, fluid, persist};

#[derive(Clone)]
pub struct Persist;

pub struct Deps {
    building: Depend<building::Persist>,
    facility: Depend<facility::Persist>,
    conduit:  Depend<conduit::Persist>,
}

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "graph:connection" }

    type Deps = Deps;
    fn depends(&self, depends: &mut impl persist::Depends) -> Self::Deps {
        Deps {
            building: depends.request(building::Persist),
            facility: depends.request(facility::Persist),
            conduit:  depends.request(conduit::Persist),
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
            .connection_query
            .iter()
            .map(|data| {
                Ok(Entry {
                    id:           ctx.alloc(self, data.entity),
                    main:         ctx.get_id(deps.facility, data.main.0)?,
                    peer:         if let Some(facility) = data.alt_facility {
                        EntryPeer::Facility { peer: ctx.get_id(deps.facility, facility.0)? }
                    } else if let Some(building) = data.building {
                        EntryPeer::Building { peer: ctx.get_id(deps.building, building.0)? }
                    } else if let Some(pipe) = data.pipe {
                        EntryPeer::Pipe { peer: ctx.get_id(deps.conduit, pipe.0)? }
                    } else {
                        unreachable!("Connection must have one of the peer components")
                    },
                    current_area: data.fluid_edge.area,
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

            let main = ctx
                .resolve_entity(deps.facility, entry.main)
                .map_err(|err| InputError::UnresolvedMainFacility { err })?;
            let peer = match entry.peer {
                EntryPeer::Facility { peer } => connection::SpawnPeer::Facility(
                    ctx.resolve_entity(deps.facility, peer)
                        .map_err(|err| InputError::UnresolvedAltFacility { err })?,
                ),
                EntryPeer::Building { peer } => connection::SpawnPeer::Building(
                    ctx.resolve_entity(deps.building, peer)
                        .map_err(|err| InputError::UnresolvedBuilding { err })?,
                ),
                EntryPeer::Pipe { peer } => connection::SpawnPeer::Pipe(
                    ctx.resolve_entity(deps.conduit, peer)
                        .map_err(|err| InputError::UnresolvedConduit { err })?,
                ),
            };
            entity.reborrow_scope(|entity| connection::SpawnCommand { main, peer }.apply(entity));
            if let Some(mut edge) = entity.log_get_mut::<fluid::Edge>() {
                edge.area = entry.current_area;
            }
        }
        Ok(())
    }
}

#[derive(SystemParam)]
pub struct OutputParams<'w, 's> {
    connection_query: Query<'w, 's, OutputQueryData>,
}

#[derive(QueryData)]
struct OutputQueryData {
    entity:     Entity,
    connection: &'static Connection,
    main:       &'static connection::MainFacility,
    fluid_edge: &'static fluid::Edge,

    alt_facility: Option<&'static connection::AltFacility>,
    building:     Option<&'static connection::ToBuilding>,
    pipe:         Option<&'static connection::ToPipe>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    id:           persist::Id,
    main:         persist::Id,
    peer:         EntryPeer,
    current_area: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum EntryPeer {
    Facility { peer: persist::Id },
    Building { peer: persist::Id },
    Pipe { peer: persist::Id },
}

#[derive(Debug, Snafu)]
pub enum InputError {
    #[snafu(display("Register new ID: {err}"))]
    RecordIdError { err: persist::IdError },
    #[snafu(display("Unresolved main facility: {err}"))]
    UnresolvedMainFacility { err: persist::IdError },
    #[snafu(display("Unresolved facility for connection peer: {err}"))]
    UnresolvedAltFacility { err: persist::IdError },
    #[snafu(display("Unresolved building for connection peer: {err}"))]
    UnresolvedBuilding { err: persist::IdError },
    #[snafu(display("Unresolved conduit for connection peer: {err}"))]
    UnresolvedConduit { err: persist::IdError },
}
