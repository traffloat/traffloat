use std::borrow::Cow;

use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{EntityCommand, Query, SystemParam};
use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::graph::{Conduit, ConduitType, conduit, corridor};
use crate::persist::{Depend, InputContext, OutputContext, Persistable};
use crate::util::EntityWorldMutExt;
use crate::{WorldObject, fluid, persist, view};

#[derive(Clone)]
pub struct Persist;

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "graph:conduit" }

    fn depends(&self) -> impl IntoIterator<Item = Depend> {
        [Depend::new(corridor::Persist), Depend::new(fluid::PersistTypes)]
    }

    type OutputParams<'w, 's> = OutputParams<'w, 's>;
    type Output = Vec<Entry>;

    fn output(
        &self,
        params: &mut OutputParams<'_, '_>,
        ctx: &mut OutputContext,
    ) -> Result<Self::Output, ()> {
        params
            .conduit_query
            .iter()
            .map(|data| {
                Ok(Entry {
                    id:       ctx.alloc(data.entity),
                    corridor: ctx.get_id(data.corridor.0)?,
                    name:     data.named.name.clone(),
                    radius:   data.conduit.radius,
                    ty:       data.conduit.ty,
                    fluid:    data.fluid.map(fluid::persist::StorageEntry::from_component),
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
            entity.reborrow_scope(|entity| {
                conduit::SpawnCommand {
                    corridor: ctx
                        .resolve_entity(entry.corridor)
                        .map_err(|err| InputError::UnresolvedCorridor { err })?,
                    name:     entry.name,
                    radius:   entry.radius,
                    typed:    match entry.ty {
                        ConduitType::FluidPipe => conduit::TypedSpawn::FluidPipe,
                    },
                }
                .apply(entity);
                Ok(())
            })?;

            if let Some(entry_fluid) = entry.fluid {
                let Some(mut fluid) = entity.log_get_mut::<fluid::Storage>() else {
                    return Err(InputError::FluidStorageMismatchConduitType);
                };
                entry_fluid.apply_to_component(&mut fluid);
            }
        }
        Ok(())
    }
}

#[derive(SystemParam)]
pub struct OutputParams<'w, 's> {
    conduit_query: Query<'w, 's, OutputQueryData>,
}

#[derive(QueryData)]
struct OutputQueryData {
    entity:   Entity,
    corridor: &'static conduit::OfCorridor,
    conduit:  &'static Conduit,
    named:    &'static view::Named,
    fluid:    Option<&'static fluid::Storage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id:       persist::Id,
    pub corridor: persist::Id,
    pub name:     String,
    pub radius:   f32,
    pub ty:       ConduitType,
    pub fluid:    Option<fluid::persist::StorageEntry>,
}

#[derive(Debug, Snafu)]
pub enum InputError {
    #[snafu(display("Unresolved corridor: {err}"))]
    UnresolvedCorridor { err: persist::UnresolvedIdError },
    #[snafu(display("Fluid storage not expected for this conduit type"))]
    FluidStorageMismatchConduitType,
}
