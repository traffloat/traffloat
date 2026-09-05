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

pub struct Deps {
    fluid_type:    Depend<fluid::PersistTypes>,
    reactor:       Depend<reactor::PersistTypes>,
    resident_attr: Depend<resident::PersistAttrTypes>,
}

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "graph:facility:type" }

    type Deps = Deps;
    fn depends(&self, depends: &mut impl persist::Depends) -> Deps {
        Deps {
            fluid_type:    depends.request(fluid::PersistTypes::default()),
            reactor:       depends.request(reactor::PersistTypes::default()),
            resident_attr: depends.request(resident::PersistAttrTypes::default()),
        }
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
            .types
            .iter()
            .map(|(entity, def)| Entry { id: ctx.alloc(self, entity), def: def.clone() })
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
            let entity = world.spawn((WorldObject, entry.def, Name::new("FacilityTypeDef")));
            ctx.record(self, entry.id, entity.id())
                .map_err(|err| InputError::RecordIdError { err })?;
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
pub enum InputError {
    #[snafu(display("Register new ID: {err}"))]
    RecordIdError { err: persist::IdError },
}
