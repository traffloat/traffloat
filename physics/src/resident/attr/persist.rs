use std::borrow::Cow;

use bevy::ecs::system::{Res, SystemParam};
use bevy::ecs::world::World;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::persist::{InputContext, OutputContext, Persistable};
use crate::resident::attr;

#[derive(Clone)]
pub struct Persist;

pub struct Deps;

impl Persistable for Persist {
    fn id(&self) -> impl Into<Cow<'static, str>> { "resident:attr:type" }

    type Deps = Deps;
    fn depends(&self, _depends: &mut impl crate::persist::Depends) -> Deps { Deps }

    type OutputParams<'w, 's> = OutputParams<'w>;
    type Output = Data;

    fn output(
        &self,
        _: &Deps,
        params: &mut OutputParams<'_>,
        _: &mut OutputContext,
    ) -> Result<Self::Output, ()> {
        Ok(Data {
            entries: params.types.iter().map(|(_ty, def)| Entry { def: def.clone() }).collect(),
            niches:  params
                .types
                .niches
                .iter()
                .filter_map(|(niche, &ty)| Some(NicheEntry { niche, ty: ty? }))
                .collect(),
        })
    }

    type Input = Data;
    type InputError = InputError;

    fn input(
        &self,
        _: &Deps,
        world: &mut World,
        input: Self::Input,
        _: &mut InputContext,
    ) -> Result<(), InputError> {
        let mut types = world.resource_mut::<attr::Types>();
        for entry in input.entries {
            types.push(entry.def);
        }
        for entry in input.niches {
            if !usize::try_from(entry.ty.0).is_ok_and(|v| v < types.len()) {
                return Err(InputError::InvalidNicheType { niche: entry.niche, ty: entry.ty });
            }
            types.niches[entry.niche] = Some(entry.ty);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub def: attr::TypeDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    pub entries: Vec<Entry>,
    pub niches:  Vec<NicheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheEntry {
    pub niche: attr::Niche,
    pub ty:    attr::TypeId,
}

#[derive(SystemParam)]
pub struct OutputParams<'w> {
    types: Res<'w, attr::Types>,
}

#[derive(Debug, Snafu)]
pub enum InputError {
    #[snafu(display("Niche {niche:?} has invalid type {:?}", ty.0))]
    InvalidNicheType { niche: attr::Niche, ty: attr::TypeId },
}
