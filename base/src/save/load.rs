use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use bevy::app::{self, App};
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Resource;
use bevy::ecs::world::{Command, World};

use super::{Def, Id, MsgpackFile, YamlFile};

pub(super) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) { app.init_resource::<LoaderMap>(); }
}

pub(super) fn add_def<D: Def>(app: &mut App) {
    fn do_load<D: Def>(
        world: &mut World,
        defs: Vec<D>,
        depend_source: &mut DependSource,
    ) -> Result<(), Error> {
        let loader = D::loader();
        let depends = loader
            .resolve_depends(depend_source)
            .map_err(|(_, dependency_ty)| Error::UninitDepend(type_name::<D>(), dependency_ty))?;

        let mut registry = IdRegistry::default();

        for (i, def) in defs.into_iter().enumerate() {
            let entity = loader
                .load(world, def, &depends)
                .map_err(|err| Error::ValidationError(type_name::<D>(), i, err))?;
            registry.save_id_to_entity.insert(
                i.try_into().map_err(|_| Error::RegistryOverflow(type_name::<D>()))?,
                entity,
            );
        }

        depend_source.0.insert(TypeId::of::<D>(), Arc::new(registry));

        Ok(())
    }

    fn load_yaml<D: Def>(
        world: &mut World,
        defs: serde_yaml::Value,
        depends: &mut DependSource,
    ) -> Result<(), Error> {
        let defs: Vec<D> =
            serde_yaml::from_value(defs).map_err(|err| Error::YamlDecodeType(D::TYPE, err))?;
        do_load(world, defs, depends)?;

        Ok(())
    }

    fn load_msgpack<D: Def>(
        world: &mut World,
        defs: Vec<u8>,
        depends: &mut DependSource,
    ) -> Result<(), Error> {
        let defs: Vec<D> = rmp_serde::from_slice(&defs)
            .map_err(|err| Error::MsgpackDecodeType(type_name::<D>(), err))?;
        do_load(world, defs, depends)?;

        Ok(())
    }

    app.world_mut().resource_mut::<LoaderMap>().map.insert(
        D::TYPE,
        LoaderVtable { load_msgpack: load_msgpack::<D>, load_yaml: load_yaml::<D> },
    );
}

pub struct LoadCommand {
    pub data:        Vec<u8>,
    #[allow(clippy::type_complexity)]
    pub on_complete: Box<dyn FnOnce(&mut World, LoadResult) + Send>,
}

fn process_file(buf: &[u8], world: &mut World) -> Result<(), Error> {
    let mut depends = DependSource(HashMap::new());

    if let Some(buf) = buf.strip_prefix(super::MSGPACK_HEADER) {
        let file: MsgpackFile = rmp_serde::from_slice(buf).map_err(Error::MsgpackDecodeFile)?;
        for ty in file.types {
            let loader = world
                .resource::<LoaderMap>()
                .map
                .get(ty.r#type.as_str())
                .copied()
                .ok_or_else(|| Error::UnsupportedType(ty.r#type.clone()))?;

            (loader.load_msgpack)(world, ty.defs, &mut depends)?;
        }

        Ok(())
    } else {
        let file: YamlFile = serde_yaml::from_slice(buf).map_err(Error::YamlDecodeFile)?;
        for ty in file.types {
            let loader = world
                .resource::<LoaderMap>()
                .map
                .get(ty.r#type.as_str())
                .copied()
                .ok_or_else(|| Error::UnsupportedType(ty.r#type.clone()))?;

            (loader.load_yaml)(world, ty.defs, &mut depends)?;
        }

        Ok(())
    }
}

impl Command for LoadCommand {
    fn apply(self, world: &mut World) {
        let result = process_file(&self.data, world);
        (self.on_complete)(world, result);
    }
}

#[derive(Default, Resource)]
struct LoaderMap {
    map: HashMap<&'static str, LoaderVtable>,
}

#[derive(Clone, Copy)]
struct LoaderVtable {
    load_msgpack: fn(&mut World, Vec<u8>, &mut DependSource) -> Result<(), Error>,
    load_yaml:    fn(&mut World, serde_yaml::Value, &mut DependSource) -> Result<(), Error>,
}

pub trait LoadOnce {
    type Def: Def;
    type Depends: Depends;

    fn resolve_depends(
        &self,
        source: &DependSource,
    ) -> Result<Self::Depends, (TypeId, &'static str)>;

    fn load(
        &self,
        world: &mut World,
        def: Self::Def,
        deps: &Self::Depends,
    ) -> anyhow::Result<Entity>;
}

pub struct LoadFn<D, Deps, F>(F, PhantomData<fn(D, &Deps)>);

impl<D: Def, Deps: Depends, F> LoadFn<D, Deps, F>
where
    F: Fn(&mut World, D, &Deps) -> anyhow::Result<Entity>,
{
    pub fn new(f: F) -> Self { Self(f, PhantomData) }
}

impl<D: Def, Deps: Depends, F> LoadOnce for LoadFn<D, Deps, F>
where
    F: Fn(&mut World, D, &Deps) -> anyhow::Result<Entity>,
{
    type Def = D;
    type Depends = Deps;

    fn resolve_depends(&self, source: &DependSource) -> Result<Deps, (TypeId, &'static str)> {
        Deps::resolve(source)
    }

    fn load(
        &self,
        world: &mut World,
        def: Self::Def,
        deps: &Self::Depends,
    ) -> anyhow::Result<Entity> {
        (self.0)(world, def, deps)
    }
}

pub struct Depend<D: Def> {
    id_registry: Arc<IdRegistry>,
    _ph:         PhantomData<D>,
}

impl<D: Def> Depend<D> {
    pub fn get(&self, id: Id<D>) -> Result<Entity, Error> {
        self.id_registry
            .save_id_to_entity
            .get(&id.0)
            .copied()
            .ok_or(Error::UnresolvedReference(type_name::<D>(), id.0))
    }
}

#[derive(Default)]
struct IdRegistry {
    save_id_to_entity: HashMap<u32, Entity>,
}

pub struct DependSource(HashMap<TypeId, Arc<IdRegistry>>);

impl DependSource {
    fn get<D: Def>(&self) -> Option<&Arc<IdRegistry>> { self.0.get(&TypeId::of::<D>()) }
}

pub trait Depends: Sized {
    const DEPEND_TYPES: &'static [&'static str];

    fn resolve(source: &DependSource) -> Result<Self, (TypeId, &'static str)>;
}

macro_rules! impl_depends {
    ($($T:ident),*) => {
        impl<$($T: Def),*> Depends for ($(Depend<$T>,)*) {
            const DEPEND_TYPES: &'static [&'static str] = &[
                $(
                    <$T as Def>::TYPE,
                )*
            ];

        #[allow(unused)]
            fn resolve(source: &DependSource) -> Result<Self, (TypeId, &'static str)> {
                Ok((
                    $(
                        source.get::<$T>().map(|id_registry| Depend { id_registry: id_registry.clone(), _ph: PhantomData}).ok_or((TypeId::of::<$T>(), type_name::<$T>()))?,
                    )*
                ))
            }
        }
    }
}

bevy::utils::all_tuples!(impl_depends, 0, 15, T);

pub type LoadResult = Result<(), Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("msgpack file decode: {0}")]
    MsgpackDecodeFile(rmp_serde::decode::Error),
    #[error("msgpack type {0} decode: {1}")]
    MsgpackDecodeType(&'static str, rmp_serde::decode::Error),
    #[error("yaml file decode: {0}")]
    YamlDecodeFile(serde_yaml::Error),
    #[error("yaml value {0} decode: {1}")]
    YamlDecodeType(&'static str, serde_yaml::Error),
    #[error("unsupported def entry {0:?}")]
    UnsupportedType(String),
    #[error("encountered type {0} which must be defined after {1} in save file")]
    UninitDepend(&'static str, &'static str),
    #[error("processing value {0}#{1}: {2:?}")]
    ValidationError(&'static str, usize, anyhow::Error),
    #[error("unresolved reference to {0}#{1}")]
    UnresolvedReference(&'static str, u32),
    #[error("too many defs of type {0}")]
    RegistryOverflow(&'static str),
}
