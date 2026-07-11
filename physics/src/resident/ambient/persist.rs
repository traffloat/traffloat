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

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "resident:interaction" }

    fn depends(&self) -> impl IntoIterator<Item = Depend> {
        [Depend::new(fluid::PersistTypes), Depend::new(resident::attr::Persist)]
    }

    type OutputParams<'w, 's> = OutputParams<'w>;
    type Output = Vec<Entry>;

    fn output(
        &self,
        params: &mut OutputParams<'_>,
        ctx: &mut OutputContext,
    ) -> Result<Self::Output, ()> {
        Ok(params.types.list.iter().map(|def| Entry { def: def.clone() }).collect())
    }

    type Input = Vec<Entry>;
    type InputError = InputError;

    fn input(
        &self,
        world: &mut World,
        input: Self::Input,
        ctx: &mut InputContext,
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
