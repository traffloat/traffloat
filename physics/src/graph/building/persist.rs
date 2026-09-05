use std::borrow::Cow;

use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{EntityCommand, Query, SystemParam};
use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use traffloat_util::EntityWorldMutExt;

use crate::graph::{Building, building};
use crate::persist::{Depend, InputContext, OutputContext, Persistable};
use crate::{Vector, WorldObject, fluid, persist, view};

#[derive(Clone)]
pub struct Persist;

pub struct Deps {
    fluid_type: Depend<fluid::PersistTypes>,
}

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "graph:building" }

    type Deps = Deps;
    fn depends(&self, depends: &mut impl persist::Depends) -> Deps {
        Deps { fluid_type: depends.request(fluid::PersistTypes::default()) }
    }

    type OutputParams<'w, 's> = OutputParams<'w, 's>;
    type Output = Vec<Entry>;

    fn output(
        &self,
        _: &Deps,
        params: &mut OutputParams<'_, '_>,
        ctx: &mut OutputContext,
    ) -> Result<Self::Output, ()> {
        Ok(params
            .building_query
            .iter()
            .map(|data| Entry {
                id:             ctx.alloc(self, data.entity),
                name:           data.named.name.clone(),
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
        _: &Deps,
        world: &mut World,
        input: Self::Input,
        ctx: &mut InputContext,
    ) -> Result<(), InputError> {
        for entry in input {
            let mut entity = world.spawn((WorldObject,));
            ctx.record(self, entry.id, entity.id())
                .map_err(|err| InputError::RecordIdError { err })?;

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
    named:    &'static view::Named,
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
pub enum InputError {
    #[snafu(display("Register new ID: {err}"))]
    RecordIdError { err: persist::IdError },
}
