use std::marker::PhantomData;
use std::{fmt, hash};

use dynec::{comp, Archetype};

/// A generic numeric value that describes an entity of archetype `A`.
#[comp(of = A, isotope = Id<A>)]
pub struct Attribute<A: Archetype>(pub f64, #[not_entity] pub PhantomData<A>);

/// The discriminant for an attribute component.
pub struct Id<A: Archetype>(pub usize, pub PhantomData<A>);

impl<A: Archetype> fmt::Debug for Id<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Id").field(&self.0).finish()
    }
}
impl<A: Archetype> Clone for Id<A> {
    fn clone(&self) -> Self { *self }
}
impl<A: Archetype> Copy for Id<A> {}
impl<A: Archetype> PartialEq for Id<A> {
    fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}
impl<A: Archetype> Eq for Id<A> {}
impl<A: Archetype> hash::Hash for Id<A> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) { self.0.hash(state) }
}

impl<A: Archetype> comp::Discrim for Id<A> {
    type FullMap<S> = comp::discrim::LinearVecMap<Self, S>;

    fn from_usize(usize: usize) -> Self { Self(usize, PhantomData) }

    fn into_usize(self) -> usize { self.0 }
}
