use std::borrow::Cow;

use bevy::ecs::system::{Res, SystemParam};
use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::persist::{Depend, InputContext, OutputContext, Persistable};
use crate::{fluid, reactor};

#[derive(Clone)]
pub struct Persist;

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "reactor:type" }

    fn depends(&self) -> impl IntoIterator<Item = Depend> { [Depend::new(fluid::PersistTypes)] }

    type OutputParams<'w, 's> = OutputParams<'w>;
    type Output = Vec<Entry>;

    fn output(
        &self,
        params: &mut OutputParams<'_>,
        ctx: &mut OutputContext,
    ) -> Result<Self::Output, ()> {
        Ok(params.types.iter().map(|(_ty, def)| Entry { def: def.clone() }).collect())
    }

    type Input = Vec<Entry>;
    type InputError = InputError;

    fn input(
        &self,
        world: &mut World,
        input: Self::Input,
        ctx: &mut InputContext,
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
