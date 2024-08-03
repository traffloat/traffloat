use std::any::{type_name, TypeId};
use std::marker::PhantomData;
use std::{iter, mem};

use bevy::app::{self, App};
use bevy::ecs::schedule::{
    IntoSystemConfigs, IntoSystemSetConfigs, ScheduleLabel, SystemConfigs, SystemSet,
    SystemSetConfigs,
};
use bevy::ecs::system::{IntoSystem, Res, ResMut, Resource, SystemParam};
use bevy::ecs::world::{Command, World};
use bevy::utils::HashMap;

use super::{Def, Format, Id, MsgpackFile, MsgpackTypedData, YamlFile, YamlTypedData};

pub(super) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) { app.insert_resource(GlobalWriter::Uninit); }
}

pub(super) fn add_def<D: Def>(app: &mut App) {
    app.init_schedule(Schedule::Store);
    app.init_schedule(Schedule::PostStore);

    app.insert_resource(Buffer::<D>(Vec::new()));
    app.insert_resource(IdRegistry::<D> {
        rt_to_save_id: <_>::default(),
        _ph:           PhantomData,
    });

    app.add_systems(
        Schedule::PostStore,
        (|mut global_writer: ResMut<GlobalWriter>,
          mut registry: ResMut<IdRegistry<D>>,
          mut buffer: ResMut<Buffer<D>>| {
            registry.rt_to_save_id.clear();
            global_writer.write_all(mem::take(&mut buffer.0));
        })
        .in_set(StoreSystemSet(TypeId::of::<D>())),
    );

    let store_system = D::store_system();
    app.add_systems(Schedule::Store, store_system.to_system());

    app.configure_sets(Schedule::Store, store_system.configure_sets());
    app.configure_sets(Schedule::PostStore, store_system.configure_sets());
}

/// Stores world data into a buffer.
pub struct StoreCommand {
    /// The format to serialize into.
    pub format:      Format,
    /// A closure invoked with the serialized data when ready.
    pub on_complete: Box<dyn FnOnce(&mut World, StoreResult) + Send>,
}

/// The output of a store command.
pub type StoreResult = Result<Vec<u8>, Error>;

impl Command for StoreCommand {
    fn apply(self, world: &mut World) {
        *world.resource_mut::<GlobalWriter>() = match self.format {
            Format::Yaml => GlobalWriter::YamlWriter { data: Vec::new(), errs: Vec::new() },
            Format::Msgpack => GlobalWriter::MsgpackWriter { data: Vec::new(), errs: Vec::new() },
        };

        world.run_schedule(Schedule::Store);
        world.run_schedule(Schedule::PostStore);

        let writer = mem::replace(&mut *world.resource_mut::<GlobalWriter>(), GlobalWriter::Uninit);

        let output = writer.output();

        (self.on_complete)(world, output);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
enum Schedule {
    Store,
    PostStore,
}

/// Describes how to create the save entries of a type.
///
/// Call [`StoreSystemFn::new`] to construct a new instance.
#[allow(missing_docs)]
pub trait StoreSystem {
    type Def: Def;

    fn to_system(&self) -> SystemConfigs;

    fn configure_sets(&self) -> impl IntoSystemSetConfigs;
}

/// Wraps a function that yields the save entries of the given type.
///
/// See save/tests.rs for example usage.
pub struct StoreSystemFn<D: Def, Deps, Q, F, Marker>(
    F,
    PhantomData<(fn(Writer<D>, Deps, Q), Marker)>,
);

impl<D: Def, Deps, Q, F, Marker> StoreSystemFn<D, Deps, Q, F, Marker> {
    /// Construct a `StoreSystemFn` from a system function.
    ///
    /// Due to HRTB requirements, the system must be defined as `fn xxx(...) {}` separately,
    /// instead of passing a closure.
    pub fn new(f: F) -> Self
    where
        F: IntoSystem<(), (), Marker>,
        F: Fn(Writer<D>, Deps, Q),
    {
        Self(f, PhantomData)
    }
}

impl<D, Deps, Q, F, Marker> StoreSystem for StoreSystemFn<D, Deps, Q, F, Marker>
where
    D: Def,
    Deps: Depends,
    F: IntoSystem<(), (), Marker> + Copy,
    F: Fn(Writer<D>, Deps, Q),
{
    type Def = D;

    fn to_system(&self) -> SystemConfigs {
        IntoSystem::into_system(self.0).in_set(StoreSystemSet(TypeId::of::<D>())).into_configs()
    }

    fn configure_sets(&self) -> impl IntoSystemSetConfigs {
        Deps::configure_system_set(StoreSystemSet(TypeId::of::<D>()))
    }
}

/// Writes save entries to the output.
#[derive(SystemParam)]
pub struct Writer<'w, D: Def> {
    write_buf:   ResMut<'w, Buffer<D>>,
    id_registry: ResMut<'w, IdRegistry<D>>,
}

impl<'w, D: Def> Writer<'w, D> {
    /// Writes a save entry to the output.
    ///
    /// `entity` is required for dependent types to resolve.
    pub fn write(&mut self, rt: D::Runtime, def: D) {
        let save_id = self.write_buf.0.len();
        self.write_buf.0.push(def);
        self.id_registry.add(save_id, rt);
    }

    /// Writes an iterator of save entries to the output.
    pub fn write_all(&mut self, iter: impl IntoIterator<Item = (D::Runtime, D)>) {
        struct MutExtend<'a, T>(&'a mut T);

        impl<'a, A, T: Extend<A>> Extend<A> for MutExtend<'a, T> {
            fn extend<I: IntoIterator<Item = A>>(&mut self, iter: I) { self.0.extend(iter) }
            // fn extend_one(&mut self, item: A) { self.0.extend_one(item) }
            // fn extend_reserve(&mut self, additional: usize) { self.0.extend_reserve(additional) }
        }

        let initial_save_id = self.write_buf.0.len();

        let extend = iter::zip(iter, initial_save_id..)
            .map(|((entity, def), save_id)| (def, (entity, save_id)));

        (MutExtend(&mut self.write_buf.0), MutExtend(&mut *self.id_registry)).extend(extend);
    }
}

#[derive(Resource)]
struct Buffer<D>(Vec<D>);

#[derive(Resource)]
struct IdRegistry<D: Def> {
    rt_to_save_id: HashMap<D::Runtime, usize>,
    _ph:           PhantomData<fn() -> D>,
}

impl<D: Def> IdRegistry<D> {
    fn add(&mut self, save_id: usize, rt: D::Runtime) { self.rt_to_save_id.insert(rt, save_id); }
}

impl<D: Def> Extend<(D::Runtime, usize)> for IdRegistry<D> {
    fn extend<T: IntoIterator<Item = (D::Runtime, usize)>>(&mut self, iter: T) {
        self.rt_to_save_id.extend(iter);
    }
}

/// Dependency of a store system.
///
/// Must be consistent with the dependencies of the loader.
#[derive(SystemParam)]
pub struct Depend<'w, D: Def> {
    id_registry: Res<'w, IdRegistry<D>>,
}

impl<'w, D: Def> Depend<'w, D> {
    /// Gets the save ID of an entity.
    ///
    /// Returns `None` if this entity did not get saved as an instance of `D`.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn get(&self, rt: D::Runtime) -> Option<Id<D>> {
        self.id_registry
            .rt_to_save_id
            .get(&rt)
            .map(|&save_id| Id(save_id.try_into().expect("too many items"), PhantomData))
    }

    /// Gets the save ID of an entity.
    ///
    /// # Panics
    /// Panics if this entity did not get saved as an instance of `D`.
    #[must_use]
    pub fn must_get(&self, rt: D::Runtime) -> Id<D> {
        match self.get(rt) {
            Some(id) => id,
            None => panic!("parent {} {rt:?} must have been persisted", type_name::<D>()),
        }
    }
}

/// The dependencies for a store system.
///
/// Implemented by tuples of [`Depend`].
#[allow(missing_docs)]
pub trait Depends: SystemParam {
    fn configure_system_set(system_set: impl IntoSystemSetConfigs) -> SystemSetConfigs;
}

impl Depends for () {
    fn configure_system_set(system_set: impl IntoSystemSetConfigs) -> SystemSetConfigs {
        system_set.into_configs()
    }
}

macro_rules! impl_depends {
    ($($T:ident),*) => {
        impl<'w, $($T: Def),*> Depends for ($(Depend<'w, $T>,)*) {
            fn configure_system_set(system_set: impl IntoSystemSetConfigs) -> SystemSetConfigs {
                system_set
                    $(
                        .after(StoreSystemSet(TypeId::of::<$T>()))
                    )*
            }
        }
    }
}

bevy::utils::all_tuples!(impl_depends, 1, 15, T);

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
struct StoreSystemSet(TypeId);

#[derive(Resource)]
enum GlobalWriter {
    Uninit,
    YamlWriter { data: Vec<YamlTypedData>, errs: Vec<serde_yaml::Error> },
    MsgpackWriter { data: Vec<MsgpackTypedData>, errs: Vec<rmp_serde::encode::Error> },
}

impl GlobalWriter {
    fn write_all<D: Def>(&mut self, objects: Vec<D>) {
        match self {
            Self::Uninit => panic!("write_all should not be called when world is not saving"),
            Self::YamlWriter { data, errs } => match serde_yaml::to_value(&objects) {
                Ok(defs) => data.push(YamlTypedData { r#type: D::TYPE.into(), defs }),
                Err(err) => errs.push(err),
            },
            Self::MsgpackWriter { data, errs } => match rmp_serde::to_vec_named(&objects) {
                Ok(defs) => data.push(MsgpackTypedData { r#type: D::TYPE.into(), defs }),
                Err(err) => errs.push(err),
            },
        }
    }

    fn output(self) -> Result<Vec<u8>, Error> {
        match self {
            Self::Uninit => panic!("output should not be called when world is not saving"),
            Self::YamlWriter { data, errs } => {
                if !errs.is_empty() {
                    return Err(Error::YamlDefToValue(errs));
                }

                let mut buf = Vec::from(super::YAML_HEADER);
                serde_yaml::to_writer(&mut buf, &YamlFile { types: data })
                    .map_err(Error::YamlEncodeValue)?;
                Ok(buf)
            }
            Self::MsgpackWriter { data, errs } => {
                if !errs.is_empty() {
                    return Err(Error::MsgpackEncodeDef(errs));
                }

                let mut buf = Vec::from(super::MSGPACK_HEADER);
                rmp_serde::encode::write_named(
                    &mut flate2::write::DeflateEncoder::new(&mut buf, flate2::Compression::best()),
                    &MsgpackFile { types: data },
                )
                .map_err(Error::MsgpackEncodeFile)?;

                Ok(buf)
            }
        }
    }
}

/// Error types during storing.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("transforming objects into YAML values: {0:?}")]
    YamlDefToValue(Vec<serde_yaml::Error>),
    #[error("encoding objects into YAML string: {0}")]
    YamlEncodeValue(serde_yaml::Error),
    #[error("encoding objects into msgpack: {0:?}")]
    MsgpackEncodeDef(Vec<rmp_serde::encode::Error>),
    #[error("producing msgpack file: {0}")]
    MsgpackEncodeFile(rmp_serde::encode::Error),
}
