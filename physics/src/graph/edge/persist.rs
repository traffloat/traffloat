use std::borrow::Cow;

use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{EntityCommand, Query, SystemParam};
use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use traffloat_proto::proto::AlphaOrBeta;
use traffloat_util::{Alpha, Beta, Which};

use crate::graph::{Edge, building, corridor, edge};
use crate::persist::{Depend, InputContext, OutputContext, Persistable};
use crate::{WorldObject, persist};

#[derive(Clone)]
pub struct Persist;

pub struct Deps {
    building: Depend<building::Persist>,
    corridor: Depend<corridor::Persist>,
}

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "graph:edge" }

    type Deps = Deps;
    fn depends(&self, depends: &mut impl persist::Depends) -> Self::Deps {
        Deps {
            building: depends.request(building::Persist),
            corridor: depends.request(corridor::Persist),
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
        fn run<Ab: Which>(
            output: &mut Vec<Entry>,
            params: &mut OutputParams<'_, '_>,
            deps: &Deps,
            ctx: &mut OutputContext,
            select: impl for<'a, 'w, 's> FnOnce(
                &'a mut Query<'w, 's, OutputQueryData<Alpha>>,
                &'a mut Query<'w, 's, OutputQueryData<Beta>>,
            )
                -> &'a mut Query<'w, 's, OutputQueryData<Ab>>,
        ) -> Result<(), ()> {
            let edge_query = select(&mut params.edge_alpha_query, &mut params.edge_beta_query);
            for data in edge_query {
                output.push(Entry {
                    id:       ctx.alloc(&Persist, data.entity),
                    building: ctx.get_id(deps.building, data.building.building)?,
                    corridor: ctx.get_id(deps.corridor, data.corridor.0)?,
                    which:    Ab::default().proto(),
                    open:     data.edge.open,
                });
            }
            Ok(())
        }

        let mut output = Vec::new();
        run(&mut output, params, deps, ctx, |a, b| a)?;
        run(&mut output, params, deps, ctx, |a, b| b)?;
        Ok(output)
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
            fn spawn_command<Ab: Which>(
                deps: &Deps,
                ctx: &InputContext,
                entry: Entry,
                which: Ab,
            ) -> Result<edge::SpawnCommand<Ab>, InputError> {
                Ok(edge::SpawnCommand {
                    building: ctx
                        .resolve_entity(deps.building, entry.building)
                        .map_err(|err| InputError::UnresolvedBuilding { err })?,
                    corridor: ctx
                        .resolve_entity(deps.corridor, entry.corridor)
                        .map_err(|err| InputError::UnresolvedCorridor { err })?,
                    which,
                    open: entry.open,
                })
            }
            let entity = world.spawn((WorldObject,));
            ctx.record(self, entry.id, entity.id())
                .map_err(|err| InputError::RecordIdError { err })?;
            match entry.which {
                AlphaOrBeta::Alpha => spawn_command(deps, ctx, entry, Alpha)?.apply(entity),
                AlphaOrBeta::Beta => spawn_command(deps, ctx, entry, Beta)?.apply(entity),
            }
        }
        Ok(())
    }
}

#[derive(SystemParam)]
pub struct OutputParams<'w, 's> {
    edge_alpha_query: Query<'w, 's, OutputQueryData<Alpha>>,
    edge_beta_query:  Query<'w, 's, OutputQueryData<Beta>>,
}

#[derive(QueryData)]
struct OutputQueryData<Ab: Which> {
    entity:   Entity,
    edge:     &'static Edge,
    corridor: &'static edge::OfCorridor<Ab>,
    building: &'static edge::OfBuilding<Ab>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    id:       persist::Id,
    building: persist::Id,
    corridor: persist::Id,
    which:    AlphaOrBeta,
    open:     bool,
}

#[derive(Debug, Snafu)]
pub enum InputError {
    #[snafu(display("Register new ID: {err}"))]
    RecordIdError { err: persist::IdError },
    #[snafu(display("Unresolved building for edge: {err}"))]
    UnresolvedBuilding { err: persist::IdError },
    #[snafu(display("Unresolved corridor for edge: {err}"))]
    UnresolvedCorridor { err: persist::IdError },
}
