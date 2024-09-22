use std::any::{type_name, Any, TypeId};
use std::collections::BTreeMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;

use bevy::app::{self, App};
use bevy::ecs::system::Resource;
use bevy::ecs::world::{Command, World};
use bevy::utils::{hashbrown, HashMap};
use serde_json::value::RawValue;

use super::{Def, Id, JsonFile, MsgpackFile};

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
        let depends = match loader.resolve_depends(depend_source) {
            Ok(depends) => depends,
            Err((_, dependency_ty)) => panic!("Dependency type {dependency_ty} not resolved"),
        };

        let mut registry = IdRegistry::<D>::default();

        for (i, def) in defs.into_iter().enumerate() {
            let entity = loader
                .load(world, def, &depends)
                .map_err(|err| Error::Validation(type_name::<D>(), i, err))?;
            registry.save_id_to_rt.insert(
                i.try_into().map_err(|_| Error::RegistryOverflow(type_name::<D>()))?,
                entity,
            );
        }

        depend_source.0.insert(TypeId::of::<D>(), Arc::new(registry));

        Ok(())
    }

    fn load_json<D: Def>(
        world: &mut World,
        defs: &RawValue,
        depends: &mut DependSource,
    ) -> Result<(), Error> {
        let defs: Vec<D> =
            serde_json::from_str(defs.get()).map_err(|err| Error::JsonDecodeType(D::TYPE, err))?;
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

    fn init_depend_source<D: Def>(depends: &mut DependSource) {
        depends.0.insert(TypeId::of::<D>(), Arc::<IdRegistry<D>>::default());
    }

    {
        let mut loader_map = app.world_mut().resource_mut::<LoaderMap>();
        loader_map.map.insert(
            D::TYPE,
            LoaderVtable {
                load_msgpack:       load_msgpack::<D>,
                load_json:          load_json::<D>,
                init_depend_source: init_depend_source::<D>,
            },
        );
        loader_map.deps.insert(D::TYPE, D::loader().list_depends());
    }
}

/// Load the save file in `data` into the world.
pub struct LoadCommand {
    /// Bytes of the save file.
    pub data:        Vec<u8>,
    /// Closure to be invoked when the save has been loaded.
    pub on_complete: Box<dyn FnOnce(&mut World, LoadResult) + Send>,
}

fn process_file(buf: &[u8], world: &mut World) -> Result<(), Error> {
    fn process_step<K: Eq + Hash, T>(
        world: &mut World,
        depends: &mut DependSource,
        ty: &str,
        types: &mut HashMap<K, T>,
        load_fn: impl FnOnce(&mut World, &mut DependSource, LoaderVtable, T) -> Result<(), Error>,
    ) -> Result<(), Error>
    where
        str: hashbrown::Equivalent<K>,
    {
        let &loader =
            world.resource::<LoaderMap>().map.get(ty).expect("exec_order has nonexistent type");
        if let Some(entry) = types.remove(ty) {
            load_fn(world, depends, loader, entry)
        } else {
            // type does not exist in entry file, just populate DependSource directly.
            (loader.init_depend_source)(depends);
            Ok(())
        }
    }

    let exec_order = world.resource::<LoaderMap>().toposorted_types();
    let mut depends = DependSource(HashMap::new());

    if let Some(compressed) = buf.strip_prefix(super::MSGPACK_HEADER) {
        let file: MsgpackFile =
            rmp_serde::from_read(flate2::bufread::DeflateDecoder::new(compressed))
                .map_err(Error::MsgpackDecodeFile)?;
        let mut types: HashMap<_, _> =
            file.types.into_iter().map(|entry| (entry.r#type.clone(), entry)).collect();

        for ty in exec_order {
            process_step(world, &mut depends, ty, &mut types, |world, depends, loader, entry| {
                (loader.load_msgpack)(world, entry.defs, depends)
            })?;
        }

        Ok(())
    } else {
        let file: JsonFile = serde_json::from_slice(buf).map_err(Error::JsonDecodeFile)?;
        let mut types: HashMap<_, _> =
            file.types.into_iter().map(|entry| (entry.r#type.clone(), entry)).collect();

        for ty in exec_order {
            process_step(world, &mut depends, ty, &mut types, |world, depends, loader, entry| {
                (loader.load_json)(world, &entry.defs, depends)
            })?;
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
    deps: HashMap<&'static str, Vec<&'static str>>,
    map:  HashMap<&'static str, LoaderVtable>,
}

impl LoaderMap {
    fn toposorted_types(&self) -> Vec<&'static str> {
        #[derive(Debug, Clone, Copy)]
        enum UnvisitedState {
            Unvisited,
            Resolving,
        }

        #[derive(Debug, thiserror::Error)]
        enum ToposortError {
            #[error("Unknown key referenced from {0:?}")]
            UnknownKey(Vec<&'static str>),
            #[error("Dependency cycle: {0:?}")]
            DepCycle(Vec<&'static str>),
        }

        impl ToposortError {
            fn context(mut self, key: &'static str) -> Self {
                match &mut self {
                    ToposortError::UnknownKey(list) | ToposortError::DepCycle(list) => {
                        list.push(key);
                    }
                }
                self
            }
        }

        fn dfs(
            deps_map: &HashMap<&'static str, Vec<&'static str>>,
            unvisited: &mut BTreeMap<&'static str, UnvisitedState>,
            output: &mut Vec<&'static str>,
            key: &'static str,
        ) -> Result<(), ToposortError> {
            let Some(deps) = deps_map.get(key) else {
                return Err(ToposortError::UnknownKey(vec![key]));
            };

            {
                match unvisited.get_mut(key) {
                    Some(state @ UnvisitedState::Unvisited) => *state = UnvisitedState::Resolving,
                    Some(UnvisitedState::Resolving) => {
                        return Err(ToposortError::DepCycle(vec![key]))
                    }
                    // if key does not exist in unvisited,
                    // it must have been extracted in a previous dfs call,
                    // otherwise undefined type would have returned UnknownKey due to deps_map keys.
                    None => return Ok(()),
                }
            }

            for &dep_key in deps {
                dfs(deps_map, unvisited, output, dep_key).map_err(|err| err.context(key))?;
            }

            unvisited.remove(key);
            output.push(key);

            Ok(())
        }

        let mut output = Vec::new();

        let mut unvisited: BTreeMap<_, _> =
            self.deps.keys().map(|&key| (key, UnvisitedState::Unvisited)).collect();
        while let Some((&key, &value)) = unvisited.first_key_value() {
            assert!(
                matches!(value, UnvisitedState::Unvisited),
                "no Resolving states between dfs calls"
            );
            if let Err(err) = dfs(&self.deps, &mut unvisited, &mut output, key) {
                panic!("{err}");
            }
        }

        output
    }
}

#[derive(Clone, Copy)]
struct LoaderVtable {
    load_msgpack:       fn(&mut World, Vec<u8>, &mut DependSource) -> Result<(), Error>,
    load_json:          fn(&mut World, &RawValue, &mut DependSource) -> Result<(), Error>,
    init_depend_source: fn(&mut DependSource),
}

/// Describes how to load a definition.
///
/// Call [`LoadFn::new`] to construct a new instance.
#[allow(missing_docs, clippy::missing_errors_doc)]
pub trait LoadOnce {
    /// The save entry type for this system.
    type Def: Def;
    /// The dependency types required by this loader.
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
    ) -> anyhow::Result<<Self::Def as Def>::Runtime>;

    fn list_depends(&self) -> Vec<&'static str>;
}

/// Wraps a function that updates a world with definition objects.
///
/// See save/tests.rs for example usage.
pub struct LoadFn<D, Deps, F>(F, PhantomData<fn(D, &Deps)>);

impl<D: Def, Deps: Depends, F> LoadFn<D, Deps, F>
where
    F: Fn(&mut World, D, &Deps) -> anyhow::Result<D::Runtime>,
{
    /// Construct a `LoadFn` from a function.
    ///
    /// The function should return an entity corresponding to this save entry.
    /// It should return an error (instead of panicking) if the save data are invalid.
    pub fn new(f: F) -> Self { Self(f, PhantomData) }
}

impl<D: Def, Deps: Depends, F> LoadOnce for LoadFn<D, Deps, F>
where
    F: Fn(&mut World, D, &Deps) -> anyhow::Result<D::Runtime>,
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
    ) -> anyhow::Result<D::Runtime> {
        (self.0)(world, def, deps)
    }

    fn list_depends(&self) -> Vec<&'static str> { Deps::DEPEND_TYPES.to_vec() }
}

/// Dependency of a loader.
///
/// Must be consistent with the dependencies of the store system.
pub struct Depend<D: Def> {
    id_registry: Arc<IdRegistry<D>>,
    _ph:         PhantomData<D>,
}

impl<D: Def> Depend<D> {
    /// Retrieves the entity for a dependency type.
    ///
    /// # Errors
    /// Returns an error if the referenced ID did not exist.
    pub fn get(&self, id: Id<D>) -> Result<D::Runtime, Error> {
        self.id_registry
            .save_id_to_rt
            .get(&id.0)
            .copied()
            .ok_or(Error::UnresolvedReference(type_name::<D>(), id.0))
    }
}

struct IdRegistry<D: Def> {
    save_id_to_rt: HashMap<u32, D::Runtime>,
}

impl<D: Def> Default for IdRegistry<D> {
    fn default() -> Self { Self { save_id_to_rt: HashMap::new() } }
}

pub struct DependSource(HashMap<TypeId, Arc<dyn Any + Send + Sync>>);

impl DependSource {
    fn get<D: Def>(&self) -> Option<Arc<IdRegistry<D>>> {
        let arc = self.0.get(&TypeId::of::<D>())?;
        Some(Arc::downcast::<IdRegistry<D>>(arc.clone()).expect("TypeId mismatch"))
    }
}

/// The dependencies for a store system.
///
/// Implemented by tuples of [`Depend`].
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
                        source.get::<$T>().map(|id_registry| Depend { id_registry, _ph: PhantomData}).ok_or((TypeId::of::<$T>(), type_name::<$T>()))?,
                    )*
                ))
            }
        }
    }
}

bevy::utils::all_tuples!(impl_depends, 0, 15, T);

/// Result of a load command.
pub type LoadResult = Result<(), Error>;

/// Error types during loading.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("msgpack file decode: {0}")]
    MsgpackDecodeFile(rmp_serde::decode::Error),
    #[error("msgpack type {0} decode: {1}")]
    MsgpackDecodeType(&'static str, rmp_serde::decode::Error),
    #[error("json file decode: {0}")]
    JsonDecodeFile(serde_json::Error),
    #[error("json value {0} decode: {1}")]
    JsonDecodeType(&'static str, serde_json::Error),
    #[error("unsupported def entry {0:?}")]
    UnsupportedType(String),
    #[error("processing value {0}#{1}: {2:?}")]
    Validation(&'static str, usize, anyhow::Error),
    #[error("unresolved reference to {0}#{1}")]
    UnresolvedReference(&'static str, u32),
    #[error("too many defs of type {0}")]
    RegistryOverflow(&'static str),
}
