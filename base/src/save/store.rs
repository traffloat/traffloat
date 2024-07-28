use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::{iter, mem};

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::ecs::schedule::{
    IntoSystemConfigs, IntoSystemSetConfigs, ScheduleLabel, SystemConfigs, SystemSet,
    SystemSetConfigs,
};
use bevy::ecs::system::{IntoSystem, Res, ResMut, Resource, SystemParam};
use bevy::ecs::world::{Command, World};
use itertools::Itertools;

use super::{Def, Id, ProtobufTypedData, YamlTypedData};

pub(super) fn add_def<D: Def>(app: &mut App) {
    app.add_systems(
        Schedule::PostWrite,
        |mut global_writer: ResMut<GlobalWriter>,
         mut registry: ResMut<IdRegistry<D>>,
         mut buffer: ResMut<Buffer<D>>| {
            registry.entity_to_save_id.clear();
            global_writer.write_all(mem::take(&mut buffer.0));
        },
    );

    let store_system = D::store_system();
    app.add_systems(Schedule::Write, store_system.to_system());

    app.configure_sets(Schedule::Write, store_system.configure_sets());
}

pub struct StoreCommand;

impl Command for StoreCommand {
    fn apply(self, world: &mut World) {
        world.run_schedule(Schedule::PreWrite);
        world.run_schedule(Schedule::Write);
        world.run_schedule(Schedule::PostWrite);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub enum Schedule {
    PreWrite,
    Write,
    PostWrite,
}

pub trait StoreSystem {
    type Def: Def;

    fn to_system(&self) -> SystemConfigs;

    fn configure_sets(&self) -> impl IntoSystemSetConfigs;
}

#[allow(clippy::type_complexity)]
pub struct SystemFn<D: Def, Deps, Q, F, Marker>(F, PhantomData<(fn(Writer<D>, Deps, Q), Marker)>);

impl<D: Def, Deps, Q, F, Marker> SystemFn<D, Deps, Q, F, Marker> {
    pub fn new(f: F) -> Self
    where
        F: IntoSystem<(), (), Marker>,
        F: Fn(Writer<D>, Deps, Q),
    {
        Self(f, PhantomData)
    }
}

impl<D, Deps, Q, F, Marker> StoreSystem for SystemFn<D, Deps, Q, F, Marker>
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

#[derive(SystemParam)]
pub struct Writer<'w, D: Def> {
    write_buf:   ResMut<'w, Buffer<D>>,
    id_registry: ResMut<'w, IdRegistry<D>>,
}

impl<'w, D: Def> Writer<'w, D> {
    pub fn write(&mut self, entity: Entity, def: D) {
        let save_id = self.write_buf.0.len();
        self.write_buf.0.push(def);
        self.id_registry.add(save_id, entity);
    }

    pub fn write_all(&mut self, iter: impl IntoIterator<Item = (Entity, D)>) {
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
struct IdRegistry<D> {
    entity_to_save_id: HashMap<Entity, usize>,
    _ph:               PhantomData<fn() -> D>,
}

impl<D> IdRegistry<D> {
    fn add(&mut self, save_id: usize, entity: Entity) {
        self.entity_to_save_id.insert(entity, save_id);
    }
}

impl<D> Extend<(Entity, usize)> for IdRegistry<D> {
    fn extend<T: IntoIterator<Item = (Entity, usize)>>(&mut self, iter: T) {
        self.entity_to_save_id.extend(iter);
    }
}

#[derive(SystemParam)]
pub struct Depend<'w, D: Def> {
    id_registry: Res<'w, IdRegistry<D>>,
}

impl<'w, D: Def> Depend<'w, D> {
    pub fn get(&self, entity: Entity) -> Option<Id<D>> {
        self.id_registry.entity_to_save_id.get(&entity).map(|&save_id| Id(save_id, PhantomData))
    }
}

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

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
struct StoreSystemSet(TypeId);

bevy::utils::all_tuples!(impl_depends, 1, 15, T);

#[derive(Resource)]
enum GlobalWriter {
    YamlWriter { data: Vec<YamlTypedData>, errs: Vec<serde_yaml::Error> },
    ProtobufWriter(Vec<ProtobufTypedData>),
}

impl GlobalWriter {
    fn write_all<D: Def>(&mut self, objects: Vec<D>) {
        match self {
            Self::YamlWriter { data, errs } => {
                let (defs, new_errs) = objects
                    .into_iter()
                    .map(|object| serde_yaml::to_value(object))
                    .partition_result();

                data.push(YamlTypedData { r#type: D::TYPE.into(), defs });
                errs.extend::<Vec<serde_yaml::Error>>(new_errs);
            }
            Self::ProtobufWriter(data) => {
                let defs = objects
                    .into_iter()
                    .map(|object| {
                        let value = object.encode_to_vec();
                        prost_types::Any {
                            type_url: format!("traffloat.github.io/{}", D::TYPE),
                            value,
                        }
                    })
                    .collect();

                data.push(ProtobufTypedData { r#type: D::TYPE.into(), defs });
            }
        }
    }
}
