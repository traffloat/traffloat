use std::borrow::Cow;

use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{EntityCommand, Query, SystemParam};
use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::graph::{Building, building};
use crate::persist::{Depend, InputContext, OutputContext, Persistable};
use crate::util::EntityWorldMutExt;
use crate::{Vector, WorldObject, fluid, persist};

#[derive(Clone)]
pub struct Persist;

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "graph:building" }

    fn depends(&self) -> impl IntoIterator<Item = Depend> { [Depend::new(fluid::PersistTypes)] }

    type OutputParams<'w, 's> = OutputParams<'w, 's>;
    type Output = Vec<Entry>;

    fn output(
        &self,
        params: &mut OutputParams<'_, '_>,
        ctx: &mut OutputContext,
    ) -> Result<Self::Output, ()> {
        Ok(params
            .building_query
            .iter()
            .map(|data| Entry {
                id:             ctx.alloc(data.entity),
                name:           data.building.name.clone(),
                position:       data.building.position,
                radius:         data.building.radius,
                wall_thickness: data.building.wall_thickness,
                fluid:          fluid::persist::StorageEntry::from_component(data.fluid),
            })
            .collect())
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
                building::SpawnCommand {
                    name:           entry.name,
                    position:       entry.position,
                    radius:         entry.radius,
                    wall_thickness: entry.wall_thickness,
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
    building_query: Query<'w, 's, OutputQueryData>,
}

#[derive(QueryData)]
struct OutputQueryData {
    entity:   Entity,
    building: &'static Building,
    fluid:    &'static fluid::Storage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id:             persist::Id,
    pub name:           String,
    pub position:       Vector,
    pub radius:         f32,
    pub wall_thickness: f32,
    pub fluid:          fluid::persist::StorageEntry,
}

#[derive(Debug, Snafu)]
pub enum InputError {}
