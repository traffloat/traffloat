//! Generic system ordering management utils.

use bevy::app::{self, App};
use bevy::ecs::event::Event;
use bevy::ecs::schedule::IntoSystemSetConfigs;

/// Declares a generic system set that takes a type argument.
#[macro_export]
macro_rules! generic_system_set {
    ($(#[$meta:meta])* $vis:vis $name:ident<$($param:ident $(: $bounds:path)?),*>) => {
        $(#[$meta])*
        #[derive(bevy::ecs::schedule::SystemSet)]
        pub struct $name<$($param $(: $bounds)?),*>(std::marker::PhantomData<T>);

        impl<$($param $(: $bounds)?),*> Default for $name<$($param),*> {
            fn default() -> Self {
                Self(std::marker::PhantomData)
            }
        }

        impl<$($param $(: $bounds)?),*> std::fmt::Debug for $name<$($param),*> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(concat!(stringify!($name), "<", $(stringify!($param),)* ">")).finish()
            }
        }

        #[allow(clippy::expl_impl_clone_on_copy)] // I can't...
        impl<$($param $(: $bounds)?),*> Clone for $name<$($param),*> {
            fn clone(&self) -> Self { *self }
        }

        impl<$($param $(: $bounds)?),*> Copy for $name<$($param),*> {}

        impl<$($param $(: $bounds)?),*> PartialEq for $name<$($param),*> {
            fn eq(&self, _: &Self) -> bool { true }
        }

        impl<$($param $(: $bounds)?),*> Eq for $name<$($param),*> {}

        impl<$($param $(: $bounds)?),*> std::hash::Hash for $name<$($param),*> {
            fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
        }
    }
}

generic_system_set!(
/// Systems producing an event type.
pub EventWriterSystemSet<T: Event>);

generic_system_set!(
/// Systems subscribing to an event type.
pub EventReaderSystemSet<T: Event>);

/// Adds partitioning methods to [`App`].
pub trait AppExt {
    /// Registers an event and its partitioning system sets.
    fn add_partitioned_event<T: Event>(&mut self);
}

impl AppExt for App {
    fn add_partitioned_event<T: Event>(&mut self) {
        self.add_event::<T>();
        self.configure_sets(
            app::Update,
            EventReaderSystemSet::<T>::default().after(EventWriterSystemSet::<T>::default()),
        );
    }
}
