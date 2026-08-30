use std::borrow::Cow;

use bevy::ecs::system::{Res, SystemParam};
use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::persist::{Depend, InputContext, OutputContext, Persistable};
use crate::resident::ambient;
use crate::{fluid, resident};

#[derive(Clone)]
pub struct Persist;

pub struct Deps {
    fluid_type:    Depend<fluid::PersistTypes>,
    resident_attr: Depend<resident::attr::Persist>,
}

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "resident:interaction" }

    type Deps = Deps;
    fn depends(&self, depends: &mut impl crate::persist::Depends) -> Self::Deps {
        Deps {
            fluid_type:    depends.request(fluid::PersistTypes),
            resident_attr: depends.request(resident::attr::Persist),
        }
    }

    type OutputParams<'w, 's> = OutputParams<'w>;
    type Output = Vec<Entry>;

    fn output(
        &self,
        _: &Deps,
        params: &mut OutputParams<'_>,
        _: &mut OutputContext,
    ) -> Result<Self::Output, ()> {
        Ok(params.types.list.iter().map(|def| Entry { def: def.clone() }).collect())
    }

    type Input = Vec<Entry>;
    type InputError = InputError;

    fn input(
        &self,
        _: &Deps,
        world: &mut World,
        input: Self::Input,
        _: &mut InputContext,
    ) -> Result<(), InputError> {
        let mut types = world.resource_mut::<ambient::Interactions>();
        for entry in input {
            types.list.push(entry.def);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub def: ambient::Interaction,
}

#[derive(SystemParam)]
pub struct OutputParams<'w> {
    types: Res<'w, ambient::Interactions>,
}

#[derive(Debug, Snafu)]
pub enum InputError {}
