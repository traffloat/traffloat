use std::borrow::Cow;

use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{EntityCommand, Query, SystemParam};
use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::graph::{Corridor, corridor};
use crate::persist::{Depend, InputContext, OutputContext, Persistable};
use crate::util::{AlphaBeta, EntityWorldMutExt};
use crate::{Vector, WorldObject, fluid, persist, view};

#[derive(Clone)]
pub struct Persist;

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "graph:corridor" }

    fn depends(&self) -> impl IntoIterator<Item = Depend> { [Depend::new(fluid::PersistTypes)] }

    type OutputParams<'w, 's> = OutputParams<'w, 's>;
    type Output = Vec<Entry>;

    fn output(
        &self,
        params: &mut OutputParams<'_, '_>,
        ctx: &mut OutputContext,
    ) -> Result<Self::Output, ()> {
        params
            .corridor_query
            .iter()
            .map(|data| {
                Ok(Entry {
                    id:                 ctx.alloc(data.entity),
                    name:               data.named.name.clone(),
                    radius:             data.corridor.radius,
                    wall_thickness:     data.corridor.wall_thickness,
                    endpoint_positions: data.corridor.endpoint_positions,
                    fluid:              fluid::persist::StorageEntry::from_component(data.fluid),
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
                corridor::SpawnCommand {
                    name:               Some(entry.name),
                    radius:             entry.radius,
                    wall_thickness:     entry.wall_thickness,
                    endpoint_positions: entry.endpoint_positions,
                }
                .apply(entity);
            });

            if let Some(mut fluid) = entity.log_get_mut::<fluid::Storage>() {
                entry.fluid.apply_to_component(&mut fluid);
            }
        }
        Ok(())
    }
}

#[derive(SystemParam)]
pub struct OutputParams<'w, 's> {
    corridor_query: Query<'w, 's, OutputQueryData>,
}

#[derive(QueryData)]
struct OutputQueryData {
    entity:   Entity,
    corridor: &'static Corridor,
    named:    &'static view::Named,
    fluid:    &'static fluid::Storage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id:                 persist::Id,
    pub name:               String,
    pub radius:             f32,
    pub wall_thickness:     f32,
    pub endpoint_positions: AlphaBeta<Vector>,
    pub fluid:              fluid::persist::StorageEntry,
}

#[derive(Debug, Snafu)]
pub enum InputError {}
