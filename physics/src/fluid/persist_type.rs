use std::borrow::Cow;

use bevy::ecs::system::{Command, Res, SystemParam};
use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::persist::{InputContext, OutputContext, Persistable};
use crate::{fluid, persist};

#[derive(Clone)]
pub struct Persist;

pub struct Deps;

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "fluid:type" }

    type Deps = Deps;
    fn depends(&self, _: &mut impl persist::Depends) -> Deps { Deps }

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
        for entry in input {
            fluid::AddTypeCommand { def: entry.def }.apply(world);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub def: fluid::TypeDef,
}

#[derive(SystemParam)]
pub struct OutputParams<'w> {
    types: Res<'w, fluid::Types>,
}

#[derive(Debug, Snafu)]
pub enum InputError {}
