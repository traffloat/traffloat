use std::borrow::Cow;

use bevy::ecs::system::{Res, SystemParam};
use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::persist::{self, Depend, InputContext, OutputContext, Persistable};
use crate::{fluid, vehicle};

#[derive(Clone)]
pub struct Persist;

pub struct Deps {
    fluid_type: Depend<fluid::PersistTypes>,
    // cargo_type: Depend<cargo::PersistTypes>,
}

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "vehicle:type" }

    type Deps = Deps;
    fn depends(&self, depends: &mut impl persist::Depends) -> Deps {
        Deps {
            fluid_type: depends.request(fluid::PersistTypes),
            // cargo_type: depends.request(cargo::PersistTypes),
        }
    }

    type OutputParams<'w, 's> = OutputParams<'w>;
    type Output = Vec<Entry>;

    fn output(
        &self,
        deps: &Deps,
        params: &mut OutputParams<'_>,
        ctx: &mut OutputContext,
    ) -> Result<Self::Output, ()> {
        Ok(params.types.iter().map(|(_ty, def)| Entry { def: def.clone() }).collect())
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
            vehicle::AddTypeCommand { def: entry.def }.run(world);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub def: vehicle::TypeDef,
}

#[derive(SystemParam)]
pub struct OutputParams<'w> {
    types: Res<'w, vehicle::Types>,
}

#[derive(Debug, Snafu)]
pub enum InputError {}
