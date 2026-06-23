use std::borrow::Cow;

use bevy::ecs::entity::Entity;
use bevy::ecs::name::Name;
use bevy::ecs::system::{Query, SystemParam};
use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::graph::FacilityTypeDef;
use crate::persist::{Depend, InputContext, OutputContext, Persistable};
use crate::{WorldObject, fluid, persist, reactor, resident};

#[derive(Clone)]
pub struct Persist;

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "graph:facility:type" }

    fn depends(&self) -> impl IntoIterator<Item = Depend> {
        [
            Depend::new(fluid::PersistTypes),
            Depend::new(reactor::Persist),
            Depend::new(resident::attr::Persist),
        ]
    }

    type OutputParams<'w, 's> = OutputParams<'w, 's>;
    type Output = Vec<Entry>;

    fn output(
        &self,
        params: &mut OutputParams<'_, '_>,
        ctx: &mut OutputContext,
    ) -> Result<Self::Output, ()> {
        Ok(params
            .types
            .iter()
            .map(|(entity, def)| Entry { id: ctx.alloc(entity), def: def.clone() })
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
            let entity = world.spawn((WorldObject, entry.def, Name::new("FacilityTypeDef")));
            ctx.record(entry.id, entity.id());
        }
        Ok(())
    }
}

#[derive(SystemParam)]
pub struct OutputParams<'w, 's> {
    types: Query<'w, 's, (Entity, &'static FacilityTypeDef)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id:  persist::Id,
    pub def: FacilityTypeDef,
}

#[derive(Debug, Snafu)]
pub enum InputError {}
