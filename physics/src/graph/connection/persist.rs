use std::borrow::Cow;

use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{EntityCommand, Query, SystemParam};
use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::graph::{Connection, building, conduit, connection, facility};
use crate::persist::{Depend, InputContext, OutputContext, Persistable};
use crate::util::EntityWorldMutExt;
use crate::{WorldObject, fluid, persist};

#[derive(Clone)]
pub struct Persist;

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "graph:connection" }

    fn depends(&self) -> impl IntoIterator<Item = Depend> {
        [
            Depend::new(building::Persist),
            Depend::new(facility::Persist),
            Depend::new(conduit::Persist),
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
            .connection_query
            .iter()
            .map(|data| {
                Ok(Entry {
                    id:           ctx.alloc(data.entity),
                    main:         ctx.get_id(data.main.0)?,
                    peer:         if let Some(facility) = data.alt_facility {
                        EntryPeer::Facility { peer: ctx.get_id(facility.0)? }
                    } else if let Some(building) = data.building {
                        EntryPeer::Building { peer: ctx.get_id(building.0)? }
                    } else if let Some(pipe) = data.pipe {
                        EntryPeer::Pipe { peer: ctx.get_id(pipe.0)? }
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
        world: &mut World,
        input: Self::Input,
        ctx: &mut InputContext,
    ) -> Result<(), InputError> {
        for entry in input {
            let mut entity = world.spawn((WorldObject,));
            ctx.record(entry.id, entity.id());

            let main = ctx
                .resolve_entity(entry.main)
                .map_err(|err| InputError::UnresolvedMainFacility { err })?;
            let peer = match entry.peer {
                EntryPeer::Facility { peer } => connection::SpawnPeer::Facility(
                    ctx.resolve_entity(peer)
                        .map_err(|err| InputError::UnresolvedAltFacility { err })?,
                ),
                EntryPeer::Building { peer } => connection::SpawnPeer::Building(
                    ctx.resolve_entity(peer)
                        .map_err(|err| InputError::UnresolvedBuilding { err })?,
                ),
                EntryPeer::Pipe { peer } => connection::SpawnPeer::Pipe(
                    ctx.resolve_entity(peer)
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
    #[snafu(display("Unresolved main facility: {err}"))]
    UnresolvedMainFacility { err: persist::UnresolvedIdError },
    #[snafu(display("Unresolved facility for connection peer: {err}"))]
    UnresolvedAltFacility { err: persist::UnresolvedIdError },
    #[snafu(display("Unresolved building for connection peer: {err}"))]
    UnresolvedBuilding { err: persist::UnresolvedIdError },
    #[snafu(display("Unresolved conduit for connection peer: {err}"))]
    UnresolvedConduit { err: persist::UnresolvedIdError },
}
