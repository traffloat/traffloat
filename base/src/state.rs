#![allow(clippy::module_name_repetitions)]

use bevy::state::state::States;

/// Declares a generic state that wraps `$wrapped` without using the generic type params.
#[macro_export]
macro_rules! generic_state {
    ($vis:vis $name:ident<$($param:ident $(: $bounds:path)?),*>($wrapped:ident)) => {
        #[derive(bevy::state::state::States)]
        pub struct $name<$($param $(: $bounds)?),*> {
            value: $wrapped,
            _ph: std::marker::PhantomData<fn($($param),*) -> $($param,)*>,
        }

        impl<$($param $(: $bounds)?),*> Default for $name<$($param),*> {
            fn default() -> Self {
                Self { value: <_>::default(), _ph: std::marker::PhantomData }
            }
        }

        impl<$($param $(: $bounds)?),*> std::fmt::Debug for $name<$($param),*> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name)).field("value", &self.value).finish()
            }
        }

        #[allow(clippy::expl_impl_clone_on_copy)] // I can't...
        impl<$($param $(: $bounds)?),*> Clone for $name<$($param),*> {
            fn clone(&self) -> Self { *self }
        }

        impl<$($param $(: $bounds)?),*> Copy for $name<$($param),*> {}

        impl<$($param $(: $bounds)?),*> PartialEq for $name<$($param),*> {
            fn eq(&self, other: &Self) -> bool {
                self.value == other.value
            }
        }

        impl<$($param $(: $bounds)?),*> Eq for $name<$($param),*> {}

        impl<$($param $(: $bounds)?),*> std::hash::Hash for $name<$($param),*> {
            fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
                std::hash::Hash::hash(&self.value, hasher);
            }
        }

        impl<$($param $(: $bounds)?),*> From<$wrapped> for $name<$($param),*> {
            fn from(value: $wrapped) -> Self {
                Self { value, _ph: std::marker::PhantomData }
            }
        }
    }
}

/// An empty state with only one value.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub struct EmptyState;
