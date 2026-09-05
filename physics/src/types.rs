//! Utils for defining runtime-loaded type lists.

use std::borrow::Cow;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;

use bevy::app::App;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Res, SystemParam};
use bevy::ecs::world::World;
use bevy::reflect::Reflect;
use derivative::Derivative;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::CleanupAppExt;
use crate::persist::{self, AppExt, InputContext, OutputContext, Persistable};

macro_rules! define_type_ {
    (
        $kind:literal, $persist_id:literal, $TypeDef:ty;
        $TypeId:ident, $PersistDeps:ident, $Types:ident, $PersistTypes:ident, $TypesGeneration:ident;
        depends { $($dep_name:ident: $dep_type:ty),* $(,)? }
    ) => {
        $crate::types::define_type! {
            $kind, $persist_id, $TypeDef;
            $TypeId, $PersistDeps, $Types, $PersistTypes, $TypesGeneration;
            depends { $($dep_name: $dep_type),* }
            serde_impl{ $crate::types::default_serde!($TypeId); }
        }
    };
    (
        $kind:literal, $persist_id:literal, $TypeDef:ty;
        $TypeId:ident, $PersistDeps:ident, $Types:ident, $PersistTypes:ident, $TypesGeneration:ident;
        depends { $($dep_name:ident: $dep_type:ty),* $(,)? }
        serde_impl { $($serde_impl:tt)* }
    ) => {
        #[doc = concat!("Identifies a ", $kind, " type.")]
        /// Indexes [`Types`].
        ///
        /// Unlike [`Entity`], this is a stable identifier preserved exactly
        /// across network sync and persistence.
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
            serde::Serialize, serde::Deserialize, bevy::reflect::Reflect,
        )]
        pub struct $TypeId(pub u32);

        impl From<u32> for $TypeId {
            fn from(value: u32) -> Self { $TypeId(value) }
        }

        impl From<usize> for $TypeId {
            fn from(value: usize) -> Self {
                let value = u32::try_from(value).expect(concat!("Too many ", $kind, " types"));
                $TypeId(value)
            }
        }

        impl From<$TypeId> for u32 {
            fn from(value: $TypeId) -> Self { value.0 }
        }

        impl From<$TypeId> for usize {
            fn from(value: $TypeId) -> Self { usize::try_from(value.0).expect("usize >= u32") }
        }

        impl $crate::types::TypeDef for $TypeDef {
            type Id = $TypeId;

            const TYPE_KIND: &'static str = $kind;
            const PERSIST_ID: &'static str = $persist_id;

            type PersistDeps = $PersistDeps;

            #[expect(clippy::allow_attributes, reason = "only active in some macro invocations")]
            #[allow(unused_variables, reason = "conditionally empty depends list")]
            fn depends(depends: &mut impl $crate::persist::Depends) -> Self::PersistDeps {
                $PersistDeps {
                    $($dep_name: depends.request(<$dep_type>::default()),)*
                }
            }

            $($serde_impl)*
        }

        pub struct $PersistDeps {
            $(pub $dep_name: $crate::persist::Depend<$dep_type>,)*
        }

        pub type $Types = $crate::types::Types<$TypeDef>;
        pub type $PersistTypes = $crate::types::Persist<$TypeDef>;
        pub type $TypesGeneration = $crate::types::Generation<$TypeDef>;
    }
}

pub(crate) use define_type_ as define_type;

macro_rules! default_serde_ {
    ($TypeDef:ty) => {
        type ExtraData = ();
        type ExtraOutputParams<'w, 's> = ();

        fn output_extra((): &mut (), _: &mut $crate::persist::OutputContext) -> Result<(), ()> {
            Ok(())
        }

        fn input_extra(
            _: &mut World,
            (): (),
            _: &Self::PersistDeps,
            _: &mut $crate::persist::InputContext,
        ) -> Result<(), Self::InputError> {
            Ok(())
        }

        type Serialize = Self;

        fn to_serialize(&self) -> Self::Serialize { self.clone() }

        type Deserialize = Self;
        type InputError = std::convert::Infallible;

        fn from_deserialize(deser: Self::Deserialize) -> Result<Self, Self::InputError> {
            Ok(deser)
        }

        fn on_cleanup(_: &mut World) {}
    };
}

pub(crate) use default_serde_ as default_serde;

pub trait TypeDef: Sized + Send + Sync + 'static {
    type Id: fmt::Debug
        + Copy
        + Ord
        + Hash
        + Serialize
        + DeserializeOwned
        + Reflect
        + From<u32>
        + Into<u32>
        + From<usize>
        + Into<usize>;

    const TYPE_KIND: &'static str;
    const PERSIST_ID: &'static str;

    type PersistDeps;

    fn depends(depends: &mut impl persist::Depends) -> Self::PersistDeps;

    type ExtraData: Serialize + DeserializeOwned;
    type ExtraOutputParams<'w, 's>: SystemParam;

    fn output_extra(
        params: &mut <Self::ExtraOutputParams<'_, '_> as SystemParam>::Item<'_, '_>,
        ctx: &mut OutputContext,
    ) -> Result<Self::ExtraData, ()>;

    fn input_extra(
        world: &mut World,
        extra: Self::ExtraData,
        deps: &Self::PersistDeps,
        ctx: &mut InputContext,
    ) -> Result<(), Self::InputError>;

    type Serialize: Serialize;

    fn to_serialize(&self) -> Self::Serialize;

    type Deserialize: DeserializeOwned;
    type InputError: std::error::Error + Sized;

    fn from_deserialize(deser: Self::Deserialize) -> Result<Self, Self::InputError>;

    fn on_cleanup(world: &mut World);
}

#[derive(Resource)]
pub struct Types<T: TypeDef> {
    types:      Vec<T>,
    generation: Generation<T>,
}

impl<T: TypeDef> Default for Types<T> {
    fn default() -> Self { Self { types: Vec::new(), generation: Generation::default() } }
}

impl<T: TypeDef> Types<T> {
    pub fn from_types(types: Vec<T>) -> Self { Self { types, generation: Generation::default() } }

    #[must_use]
    pub fn get(&self, id: T::Id) -> &T {
        match self.types.get::<usize>(id.into()) {
            Some(def) => def,
            None => panic!("got invalid {} type reference", T::TYPE_KIND),
        }
    }

    pub fn push(&mut self, def: T) -> T::Id {
        let id = T::Id::from(self.types.len());
        self.types.push(def);
        self.incr_generation();
        id
    }

    pub fn types(&self) -> &[T] { &self.types }

    pub fn incr_generation(&mut self) {
        self.generation.counter = self.generation.counter.strict_add(1);
    }

    pub fn generation(&self) -> Generation<T> { self.generation }

    pub fn iter(&self) -> impl Iterator<Item = (T::Id, &T)> {
        self.types.iter().enumerate().map(|(id, def)| (T::Id::from(id), def))
    }

    pub fn len(&self) -> usize { self.types.len() }

    pub fn is_empty(&self) -> bool { self.types.is_empty() }

    fn cleanup_hook(world: &mut World) {
        let mut this = world.resource_mut::<Self>();
        this.types.clear();
        this.incr_generation();
        T::on_cleanup(world);
    }
}

#[derive(Derivative, Reflect)]
#[derivative(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[reflect(no_field_bounds)]
pub struct Generation<T> {
    counter: u32,
    #[reflect(ignore, default)]
    _pd:     PhantomData<T>,
}

#[derive(Derivative)]
#[derivative(Debug, Clone, Copy, Default)]
pub struct Persist<T: TypeDef>(pub PhantomData<T>);

impl<T: TypeDef> Persistable for Persist<T> {
    fn id(&self) -> impl Into<Cow<'static, str>> { T::PERSIST_ID }

    type Deps = T::PersistDeps;
    fn depends(&self, depends: &mut impl persist::Depends) -> T::PersistDeps { T::depends(depends) }

    type OutputParams<'w, 's> = (Res<'w, Types<T>>, T::ExtraOutputParams<'w, 's>);
    type Output = PersistOutput<T>;

    fn output(
        &self,
        _: &Self::Deps,
        (types, extra_params): &mut <Self::OutputParams<'_, '_> as SystemParam>::Item<'_, '_>,
        ctx: &mut OutputContext,
    ) -> Result<Self::Output, ()> {
        let entries =
            types.iter().map(|(_ty, def)| PersistOutputEntry { def: def.to_serialize() }).collect();
        let extra = T::output_extra(extra_params, ctx)?;
        Ok(PersistOutput { entries, extra })
    }

    type Input = PersistInput<T>;
    type InputError = T::InputError;

    fn input(
        &self,
        deps: &Self::Deps,
        world: &mut World,
        input: Self::Input,
        ctx: &mut InputContext,
    ) -> Result<(), T::InputError> {
        let mut types = world.resource_mut::<Types<T>>();
        for entry in input.entries {
            let def = T::from_deserialize(entry.def)?;
            types.push(def);
        }
        T::input_extra(world, input.extra, deps, ctx)?;
        Ok(())
    }
}

#[derive(Serialize)]
#[serde(bound = "")]
pub struct PersistOutput<T: TypeDef> {
    pub entries: Vec<PersistOutputEntry<T>>,
    #[serde(flatten)]
    pub extra:   T::ExtraData,
}

#[derive(Serialize)]
pub struct PersistOutputEntry<T: TypeDef> {
    pub def: T::Serialize,
}

#[derive(Deserialize)]
#[serde(bound = "")]
pub struct PersistInput<T: TypeDef> {
    pub entries: Vec<PersistInputEntry<T>>,
    #[serde(flatten)]
    pub extra:   T::ExtraData,
}

#[derive(Deserialize)]
pub struct PersistInputEntry<T: TypeDef> {
    pub def: T::Deserialize,
}

pub fn init<T: TypeDef>(app: &mut App) {
    app.register_persistable(Persist::<T>::default());
    app.add_cleanup_hook(Types::<T>::cleanup_hook);
    app.init_resource::<Types<T>>();
}
