use std::any::type_name;
use std::fmt;

use bevy::ecs::change_detection::Mut;
use bevy::ecs::component::{Component, Mutable};
use bevy::ecs::entity::Entity;
use bevy::ecs::query::{QueryData, QueryFilter};
use bevy::ecs::system::Query;
use bevy::ecs::world::{EntityRef, EntityWorldMut, World};

#[macro_export]
macro_rules! try_log {
    (
        $expr:expr,
        expect $must:literal $(
            (
                $($must_args:expr),* $(,)?
            )
        )?
        or $never:expr
    ) => {
        {
            #[allow(
                clippy::allow_attributes, clippy::question_mark,
                reason = "potentially generalizes Option and Result in generated code"
            )]
            if let Some(value) = $crate::TryLog::convert_or_log(
                $expr,
                format_args!($must, $($($must_args),*)?),
            ) {
                value
            } else {
                $never
            }
        }
    }
}

pub use try_log;

#[macro_export]
macro_rules! try_log_return {
    ($expr:expr, expect $must:literal $(, $($must_args:expr),*)? $(,)?) => {
        $crate::try_log!($expr, expect $must $(($($must_args),*))? or return)
    }
}

pub use try_log_return;

pub trait QueryExtSuper {
    type Read<'a>;
    type Write<'a>;
}

pub trait QueryExt: QueryExtSuper {
    fn log_get(&self, entity: Entity) -> Option<Self::Read<'_>>;

    fn log_get_many<const N: usize>(&self, entity: [Entity; N]) -> Option<[Self::Read<'_>; N]>;

    fn log_get_mut(&mut self, entity: Entity) -> Option<Self::Write<'_>>;

    fn log_get_many_mut<const N: usize>(
        &mut self,
        entity: [Entity; N],
    ) -> Option<[Self::Write<'_>; N]>;
}

impl<'s, D, F> QueryExtSuper for Query<'_, 's, D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    type Read<'a> = <D::ReadOnly as QueryData>::Item<'a, 's>;
    type Write<'a> = D::Item<'a, 's>;
}

impl<'s, D, F> QueryExt for Query<'_, 's, D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    fn log_get(&self, entity: Entity) -> Option<<D::ReadOnly as QueryData>::Item<'_, 's>> {
        match self.get(entity) {
            Ok(value) => Some(value),
            Err(err) => {
                bevy::log::error!("Expected {entity:?} to match query {}: {err}", type_name::<D>());
                None
            }
        }
    }

    fn log_get_many<const N: usize>(
        &self,
        entity: [Entity; N],
    ) -> Option<[<D::ReadOnly as QueryData>::Item<'_, 's>; N]> {
        match self.get_many(entity) {
            Ok(value) => Some(value),
            Err(err) => {
                bevy::log::error!("Expected {entity:?} to match query {}: {err}", type_name::<D>());
                None
            }
        }
    }

    fn log_get_mut(&mut self, entity: Entity) -> Option<D::Item<'_, 's>> {
        match self.get_mut(entity) {
            Ok(value) => Some(value),
            Err(err) => {
                bevy::log::error!("Expected {entity:?} to match query {}: {err}", type_name::<D>());
                None
            }
        }
    }

    fn log_get_many_mut<const N: usize>(
        &mut self,
        entity: [Entity; N],
    ) -> Option<[D::Item<'_, 's>; N]> {
        match self.get_many_mut(entity) {
            Ok(value) => Some(value),
            Err(err) => {
                bevy::log::error!("Expected {entity:?} to match query {}: {err}", type_name::<D>());
                None
            }
        }
    }
}

pub trait WorldExt {
    fn log_get<T: Component>(&self, entity: Entity) -> Option<&T>;

    fn log_get_mut<T: Component<Mutability = Mutable>>(
        &mut self,
        entity: Entity,
    ) -> Option<Mut<'_, T>>;
}

impl WorldExt for World {
    fn log_get<T: Component>(&self, entity: Entity) -> Option<&T> {
        if let Some(value) = self.get::<T>(entity) {
            Some(value)
        } else {
            bevy::log::error!("Expected {entity:?} to have component {}", type_name::<T>());
            None
        }
    }

    fn log_get_mut<T: Component<Mutability = Mutable>>(
        &mut self,
        entity: Entity,
    ) -> Option<Mut<'_, T>> {
        if let Some(value) = self.get_mut::<T>(entity) {
            Some(value)
        } else {
            bevy::log::error!("Expected {entity:?} to have component {}", type_name::<T>());
            None
        }
    }
}

pub trait EntityRefExt {
    fn log_get<T: Component>(&self) -> Option<&T>;
}

impl EntityRefExt for EntityRef<'_> {
    fn log_get<T: Component>(&self) -> Option<&T> {
        if let Some(value) = self.get::<T>() {
            Some(value)
        } else {
            bevy::log::error!("Expected {:?} to have component {}", self.id(), type_name::<T>());
            None
        }
    }
}

pub trait EntityWorldMutExt {
    fn log_get<T: Component>(&self) -> Option<&T>;

    fn log_get_mut<T: Component<Mutability = Mutable>>(&mut self) -> Option<Mut<'_, T>>;
}

impl EntityWorldMutExt for EntityWorldMut<'_> {
    fn log_get<T: Component>(&self) -> Option<&T> {
        if let Some(value) = self.get::<T>() {
            Some(value)
        } else {
            bevy::log::error!("Expected {:?} to have component {}", self.id(), type_name::<T>());
            None
        }
    }

    fn log_get_mut<T: Component<Mutability = Mutable>>(&mut self) -> Option<Mut<'_, T>> {
        let id = self.id(); // polonius does not like this being in the match arm
        if let Some(value) = self.get_mut::<T>() {
            Some(value)
        } else {
            bevy::log::error!("Expected {:?} to have component {}", id, type_name::<T>());
            None
        }
    }
}

/// An expression that can be used for `$expr` in [`try_log!`](crate::try_log!).
pub trait TryLog<T> {
    /// Returns the successful result as `Some`, or log the error with `must`.
    fn convert_or_log(this: Self, must: impl fmt::Display) -> Option<T>;
}

impl<T> TryLog<T> for Option<T> {
    fn convert_or_log(this: Self, must: impl fmt::Display) -> Option<T> {
        if let Some(value) = this {
            Some(value)
        } else {
            bevy::log::error!("{must}");
            None
        }
    }
}

impl<T, E: fmt::Display> TryLog<T> for Result<T, E> {
    fn convert_or_log(this: Self, must: impl fmt::Display) -> Option<T> {
        match this {
            Ok(value) => Some(value),
            Err(err) => {
                bevy::log::error!("{must}: {err}");
                None
            }
        }
    }
}
