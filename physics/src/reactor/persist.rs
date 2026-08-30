use std::borrow::Cow;

use bevy::ecs::system::{Res, SystemParam};
use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::persist::{self, Depend, InputContext, OutputContext, Persistable};
use crate::{fluid, reactor, resident};

#[derive(Clone)]
pub struct Persist;

pub struct Deps {
    fluid_type:    Depend<fluid::PersistTypes>,
    resident_attr: Depend<resident::PersistAttrTypes>,
}

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "reactor:type" }

    type Deps = Deps;
    fn depends(&self, depends: &mut impl persist::Depends) -> Deps {
        Deps {
            fluid_type:    depends.request(fluid::PersistTypes),
            resident_attr: depends.request(resident::PersistAttrTypes),
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
        Ok(params.types.iter().map(|(_ty, def)| Entry { def: def.clone() }).collect())
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
        let mut types = world.resource_mut::<reactor::Types>();
        for entry in input {
            types.push(entry.def);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub def: reactor::TypeDef,
}

#[derive(SystemParam)]
pub struct OutputParams<'w> {
    types: Res<'w, reactor::Types>,
}

#[derive(Debug, Snafu)]
pub enum InputError {}
