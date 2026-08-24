use std::any::{TypeId, type_name};
use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::Hash;
use std::io;
use std::marker::PhantomData;

use bevy::app::{App, Plugin};
use bevy::ecs::entity::{Entity, EntityHashMap};
use bevy::ecs::resource::Resource;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::World;
use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use traffloat_util::run_stateless_closure_explicit;

use crate::cleanup::execute_cleanup_hooks;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) { app.init_resource::<PersistableTypes>(); }
}

pub trait AppExt {
    fn as_app_mut(&mut self) -> &mut App;

    fn register_persistable<P: Persistable>(&mut self, persistable: P) {
        let mut types = self.as_app_mut().world_mut().resource_mut::<PersistableTypes>();
        let id = persistable.id().into();
        let old = types.types.insert(id.clone().into_owned(), Box::new(persistable));
        assert!(old.is_none(), "Duplicate persistable type registered: {:?}", id.as_ref());
        types.sorted = false;
    }
}

impl AppExt for App {
    fn as_app_mut(&mut self) -> &mut App { self }
}

#[derive(Resource, Default)]
struct PersistableTypes {
    types:  IndexMap<String, PersistableBox>,
    sorted: bool,
}

type PersistableBox = Box<dyn PersistableDyn>;

impl PersistableTypes {
    fn sort(&mut self) {
        fn dfs(
            sorted: &mut IndexMap<String, PersistableBox>,
            types: &mut IndexMap<String, PersistableBox>,
            (seed_key, seed_box): (String, PersistableBox),
        ) {
            for depend in seed_box.depends() {
                if sorted.contains_key(depend.0.as_ref()) {
                    continue;
                }
                let Some(dep_seed) = types.swap_remove(depend.0.as_ref()) else {
                    panic!("Persistable type {seed_key} depends on unknown type {}", depend.0);
                };
                dfs(sorted, types, (depend.0.into_owned(), dep_seed));
            }

            sorted.insert(seed_key, seed_box);
        }

        if self.sorted {
            return;
        }

        let mut sorted = IndexMap::new();
        while let Some(seed) = self.types.pop() {
            dfs(&mut sorted, &mut self.types, seed);
        }
        self.types = sorted;
        self.sorted = true;
    }

    fn sorted_clone(&mut self) -> Vec<(String, PersistableBox)> {
        self.sort();
        self.types.iter().map(|(k, v)| (k.clone(), v.clone_box())).collect()
    }
}

pub trait Persistable: Clone + Send + Sync + 'static {
    fn id(&self) -> impl Into<Cow<'static, str>>;

    type Deps;
    fn depends(&self, depends: &mut impl Depends) -> Self::Deps;

    type OutputParams<'w, 's>: SystemParam;
    type Output: Serialize;

    fn output(
        &self,
        depends: &Self::Deps,
        params: &mut <Self::OutputParams<'_, '_> as SystemParam>::Item<'_, '_>,
        ctx: &mut OutputContext,
    ) -> Result<Self::Output, ()>;

    type Input: DeserializeOwned;
    type InputError: std::error::Error;

    fn input(
        &self,
        depends: &Self::Deps,
        world: &mut World,
        input: Self::Input,
        ctx: &mut InputContext,
    ) -> Result<(), Self::InputError>;

    fn no_input(
        &self,
        depends: &Self::Deps,
        world: &mut World,
        ctx: &mut InputContext,
    ) -> Result<(), Self::InputError> {
        Ok(())
    }
}

#[derive(derivative::Derivative)]
#[derivative(Clone, Copy, Default)]
pub struct Depend<T>(PhantomData<T>);

#[derive(Debug, Clone)]
struct DependRef(Cow<'static, str>);

pub trait Depends {
    fn request<P: Persistable>(&mut self, p: P) -> Depend<P>;
}

struct DependsNoop;

impl Depends for DependsNoop {
    fn request<P: Persistable>(&mut self, _: P) -> Depend<P> { Depend(PhantomData) }
}

struct DependsList(Vec<DependRef>);

impl Depends for DependsList {
    fn request<P: Persistable>(&mut self, p: P) -> Depend<P> {
        self.0.push(DependRef(p.id().into()));
        Depend(PhantomData)
    }
}

/// A generic ID unique within a persistence context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Id(pub u32);

#[derive(Default)]
pub struct OutputContext {
    id_map:  EntityHashMap<OutputIdEntry>,
    next_id: u32,
}

struct OutputIdEntry {
    type_id:   TypeId,
    type_name: &'static str,
    id:        Id,
}

impl OutputContext {
    /// Allocates a new ID as a reference to some entity.
    ///
    /// The allocated ID does not necessarily need to be used in the output,
    /// and may be used in any order in the output.
    pub fn alloc<P: Persistable>(&mut self, _: &P, entity: Entity) -> Id {
        let id = Id(self.next_id);
        self.next_id += 1;
        self.id_map.insert(
            entity,
            OutputIdEntry { type_id: TypeId::of::<P>(), type_name: type_name::<P>(), id },
        );
        id
    }

    /// Gets the ID corresponding to the given entity
    /// that should have been previously allocated with [`Self::alloc`].
    pub fn get_id<P: Persistable>(&self, depend: Depend<P>, entity: Entity) -> Result<Id, ()> {
        if let Some(entry) = self.id_map.get(&entity) {
            assert_eq!(
                entry.type_id,
                TypeId::of::<P>(),
                "Entity {entity:?} was allocated with a different type ({} != {})",
                entry.type_name,
                type_name::<P>()
            );
            Ok(entry.id)
        } else {
            bevy::log::warn!("Entity {entity:?} should have been persisted first");
            Err(())
        }
    }

    /// Similar to [`get_id`], but does not check the entity type.
    ///
    /// This will be deprecated in the future in favor of union types.
    pub fn get_id_unchecked(&self, entity: Entity) -> Result<Id, ()> {
        match self.id_map.get(&entity) {
            Some(entry) => Ok(entry.id),
            None => {
                bevy::log::warn!("Entity {entity:?} should have been persisted first");
                Err(())
            }
        }
    }
}

#[derive(Default)]
pub struct InputContext {
    id_map: HashMap<Id, InputIdEntry>,
}

struct InputIdEntry {
    type_id:   TypeId,
    type_name: &'static str,
    entity:    Entity,
}

impl InputContext {
    /// Records that an [`Id`] has been spawned as an [`Entity`].
    pub fn record<P: Persistable>(&mut self, _: &P, id: Id, entity: Entity) -> Result<(), IdError> {
        let old = self.id_map.insert(
            id,
            InputIdEntry { type_id: TypeId::of::<P>(), type_name: type_name::<P>(), entity },
        );
        if let Some(old) = old {
            return Err(IdError::Duplicate { id, old: old.type_name, new: type_name::<P>() });
        }
        Ok(())
    }

    /// Gets the [`Entity`] corresponding to the given [`Id`].
    pub fn resolve_entity<P: Persistable>(
        &self,
        depend: Depend<P>,
        id: Id,
    ) -> Result<Entity, IdError> {
        match self.id_map.get(&id) {
            Some(entry) => {
                if entry.type_id == TypeId::of::<P>() {
                    Ok(entry.entity)
                } else {
                    Err(IdError::TypeMismatch {
                        id,
                        expect: type_name::<P>(),
                        actual: entry.type_name,
                    })
                }
            }
            None => Err(IdError::Unresolved { id, type_name: type_name::<P>() }),
        }
    }

    /// Similar to [`resolve_entity`], but does not check the entity type.
    ///
    /// This will be deprecated in the future in favor of union types.
    pub fn resolve_entity_unchecked(&self, id: Id) -> Result<Entity, IdError> {
        match self.id_map.get(&id) {
            Some(entry) => Ok(entry.entity),
            None => Err(IdError::Unresolved { id, type_name: "unchecked" }),
        }
    }
}

#[derive(Debug, snafu::Snafu)]
pub enum IdError {
    #[snafu(display("Savefile references unknown {type_name} ID {id:?}"))]
    Unresolved { id: Id, type_name: &'static str },
    #[snafu(display(
        "Savefile references {expect} ID {id:?} but it was previously used for {actual}"
    ))]
    TypeMismatch { id: Id, expect: &'static str, actual: &'static str },
    #[snafu(display("Savefile reuses ID {id:?} for both {old} and {new}"))]
    Duplicate { id: Id, old: &'static str, new: &'static str },
}

trait PersistableDyn: Send + Sync + 'static {
    fn id(&self) -> Cow<'static, str>;

    fn depends(&self) -> Vec<DependRef>;

    fn output(&self, world: &mut World, ctx: &mut OutputContext) -> Result<Vec<u8>, ()>;

    fn input(
        &self,
        world: &mut World,
        data: &[u8],
        ctx: &mut InputContext,
    ) -> Result<(), InputError>;

    fn no_input(&self, world: &mut World, ctx: &mut InputContext) -> Result<(), InputError>;

    fn clone_box(&self) -> PersistableBox;
}

impl<P: Persistable> PersistableDyn for P {
    fn id(&self) -> Cow<'static, str> { Persistable::id(self).into() }

    fn depends(&self) -> Vec<DependRef> {
        let mut depends = DependsList(Vec::new());
        Persistable::depends(self, &mut depends);
        depends.0
    }

    #[tracing::instrument(skip_all, fields(ty = self.id().into().as_ref()))]
    fn output(&self, world: &mut World, ctx: &mut OutputContext) -> Result<Vec<u8>, ()> {
        let depends = Persistable::depends(self, &mut DependsNoop);

        let output =
            run_stateless_closure_explicit::<<P as Persistable>::OutputParams<'_, '_>, _, _>(
                world,
                |mut param| Persistable::output(self, &depends, &mut param, ctx),
            )?;
        let mut buf = Vec::new();
        match ciborium::into_writer(&output, &mut buf) {
            Ok(()) => {
                tracing::debug!("Serialized {} bytes for {}", buf.len(), self.id().into());
                Ok(buf)
            }
            Err(err) => {
                bevy::log::error!("Error serializing data for {}: {err}", self.id().into());
                Err(())
            }
        }
    }

    #[tracing::instrument(skip_all, fields(ty = self.id().into().as_ref()))]
    fn input(
        &self,
        world: &mut World,
        data: &[u8],
        ctx: &mut InputContext,
    ) -> Result<(), InputError> {
        let depends = Persistable::depends(self, &mut DependsNoop);

        let input = ciborium::from_reader::<P::Input, _>(data)
            .map_err(|err| InputError::TypedCiborium { ty: self.id().into().to_string(), err })?;
        Persistable::input(self, &depends, world, input, ctx).map_err(|err| InputError::Typed {
            ty:  self.id().into().to_string(),
            err: Box::new(err),
        })
    }

    #[tracing::instrument(skip_all, fields(ty = self.id().into().as_ref()))]
    fn no_input(&self, world: &mut World, ctx: &mut InputContext) -> Result<(), InputError> {
        let depends = Persistable::depends(self, &mut DependsNoop);
        Persistable::no_input(self, &depends, world, ctx).map_err(|err| InputError::Typed {
            ty:  self.id().into().to_string(),
            err: Box::new(err),
        })
    }

    fn clone_box(&self) -> PersistableBox { Box::new(self.clone()) }
}

pub fn output(world: &mut World) -> Result<Vec<u8>, ()> {
    let types = {
        let mut types = world.resource_mut::<PersistableTypes>();
        types.sorted_clone()
    };

    let mut ctx = OutputContext::default();
    let mut output = Vec::new();
    for (id, persistable) in types {
        let data = persistable.output(world, &mut ctx)?;
        output.push((id, data));
    }

    let mut zstd =
        zstd::Encoder::new(Vec::new(), 3).expect("compression initialization must succeed");
    match ciborium::into_writer(&output, &mut zstd) {
        Ok(()) => match zstd.finish() {
            Ok(buf) => Ok(buf),
            Err(err) => {
                bevy::log::error!("Error compressing persist data: {err}");
                Err(())
            }
        },
        Err(err) => {
            bevy::log::error!("Error serializing persist data: {err}");
            Err(())
        }
    }
}

pub fn input(world: &mut World, raw: &[u8]) -> Result<(), InputError> {
    execute_cleanup_hooks(world);

    let types = {
        let mut types = world.resource_mut::<PersistableTypes>();
        types.sorted_clone()
    };

    let mut inputs = HashMap::new();
    let raw = zstd::Decoder::new(raw).map_err(|err| InputError::Zstd { err })?;
    let input_vec: Vec<(String, Vec<u8>)> =
        ciborium::from_reader(raw).map_err(|err| InputError::Ciborium { err })?;
    for (id, data) in input_vec {
        inputs.insert(id, data);
    }

    let mut ctx = InputContext::default();
    for (id, persistable) in types {
        if let Some(data) = inputs.get(&id) {
            persistable.input(world, data, &mut ctx)?;
        } else {
            persistable.no_input(world, &mut ctx)?;
        }
    }

    Ok(())
}

#[derive(Debug, Snafu)]
pub enum InputError {
    #[snafu(display("Error decompressing persist data: {err}"))]
    Zstd { err: io::Error },
    #[snafu(display("Error destructuring persist data: {err}"))]
    Ciborium { err: ciborium::de::Error<io::Error> },
    #[snafu(display("Data for {ty} is malformed: {err}"))]
    TypedCiborium { ty: String, err: ciborium::de::Error<io::Error> },
    #[snafu(display("Error processing {ty}: {err}"))]
    Typed { ty: String, err: Box<dyn std::error::Error> },
}
