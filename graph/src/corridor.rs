//! A link between two buildings.

use std::ops;

use bevy::app;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::{QueryData, QueryFilter, QueryItem, ROQueryItem};
use bevy::ecs::{bundle, system};
use typed_builder::TypedBuilder;

pub mod duct;

/// Maintain corridors.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, _: &mut bevy::prelude::App) {}
}

/// Components for a corridor.
#[derive(bundle::Bundle, TypedBuilder)]
#[allow(missing_docs)]
pub struct Bundle {
    pub endpoints: Endpoints,
    pub ducts:     DuctList,
    pub length:    Length,
}

/// The endpoints for a corridor.
///
/// "Alpha" and "Beta" refer to the first and second endpoints of the undirected edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Endpoint {
    /// The alpha endpoint.
    Alpha,
    /// The beta endpoint.
    Beta,
}

impl ops::Not for Endpoint {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Self::Beta => Self::Alpha,
            Self::Alpha => Self::Beta,
        }
    }
}

/// A pair of values for each endpoint of a corridor.
#[derive(Default, Debug, Clone, Copy)]
pub struct Binary<T> {
    /// The value for the alpha endpoint.
    pub alpha: T,
    /// The value for the beta endpoint.
    pub beta:  T,
}

impl<T> Binary<T> {
    /// Constructs a `Binary` from a function that maps each endpoint to a value.
    pub fn from_fn(mut f: impl FnMut(Endpoint) -> T) -> Binary<T> {
        Binary { alpha: f(Endpoint::Alpha), beta: f(Endpoint::Beta) }
    }

    /// Returns the value corresponding to the endpoint.
    pub fn into_endpoint(self, endpoint: Endpoint) -> T {
        match endpoint {
            Endpoint::Alpha => self.alpha,
            Endpoint::Beta => self.beta,
        }
    }

    /// Returns the value corresponding to the endpoint.
    pub fn as_endpoint(&self, endpoint: Endpoint) -> &T {
        match endpoint {
            Endpoint::Alpha => &self.alpha,
            Endpoint::Beta => &self.beta,
        }
    }

    /// Returns the value corresponding to the endpoint.
    pub fn as_endpoint_mut(&mut self, endpoint: Endpoint) -> &mut T {
        match endpoint {
            Endpoint::Alpha => &mut self.alpha,
            Endpoint::Beta => &mut self.beta,
        }
    }

    /// Returns the value corresponding to `first_endpoint` and to `!first_endpoint` respectively.
    pub fn as_endpoints_mut(&mut self, first_endpoint: Endpoint) -> (&mut T, &mut T) {
        match first_endpoint {
            Endpoint::Alpha => (&mut self.alpha, &mut self.beta),
            Endpoint::Beta => (&mut self.beta, &mut self.alpha),
        }
    }

    /// Converts the values to reference types.
    pub fn as_ref(&self) -> Binary<&T> { Binary { alpha: &self.alpha, beta: &self.beta } }

    /// Converts the values to reference types.
    pub fn as_mut(&mut self) -> Binary<&mut T> {
        Binary { alpha: &mut self.alpha, beta: &mut self.beta }
    }

    /// Applies the closure to each value.
    #[must_use]
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Binary<U> {
        Binary { alpha: f(self.alpha), beta: f(self.beta) }
    }

    /// Combines two `Binary`s with a tuple.
    #[must_use]
    pub fn zip<U>(self, other: impl Into<Binary<U>>) -> Binary<(T, U)> {
        let other = other.into();
        Binary { alpha: (self.alpha, other.alpha), beta: (self.beta, other.beta) }
    }

    /// Executes a function to modify each value in-place.
    pub fn each_mut(&mut self, mut f: impl FnMut(&mut T)) {
        f(&mut self.alpha);
        f(&mut self.beta);
    }

    /// Iterates over both components, equivalent to `[&alpha, &beta]`.
    pub fn iter(&self) -> impl Iterator<Item = &T> { [&self.alpha, &self.beta].into_iter() }

    /// Iterates over both components, equivalent to `[&mut alpha, &mut beta]`.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.alpha, &mut self.beta].into_iter()
    }
}

impl<T> IntoIterator for Binary<T> {
    type Item = T;
    type IntoIter = <[T; 2] as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter { [self.alpha, self.beta].into_iter() }
}

impl<T: PartialEq> Binary<T> {
    /// Finds `pat` among the operands.
    ///
    /// Returns `Some(Alpha)` if the alpha value is equal to `pat`,
    /// `Some(Beta)` if the beta value is equal but the alpha value is not,
    /// or `None` if neither is equal.
    pub fn find(&self, pat: &T) -> Option<Endpoint> {
        if self.alpha == *pat {
            Some(Endpoint::Alpha)
        } else if self.beta == *pat {
            Some(Endpoint::Beta)
        } else {
            None
        }
    }
}

impl<T> From<[T; 2]> for Binary<T> {
    fn from([alpha, beta]: [T; 2]) -> Self { Self { alpha, beta } }
}

impl<T> From<(T, T)> for Binary<T> {
    fn from((alpha, beta): (T, T)) -> Self { Self { alpha, beta } }
}

impl Binary<Entity> {
    /// Performs a bevy query on both entities.
    ///
    /// # Panics
    /// Panics if the query cannot be used on the entities in `self`.
    #[must_use]
    pub fn query<'a, D: QueryData, F: QueryFilter>(
        &self,
        query: &'a system::Query<D, F>,
    ) -> Binary<ROQueryItem<'a, D>> {
        let [alpha, beta] = query.many([self.alpha, self.beta]);
        Binary { alpha, beta }
    }
}

impl Binary<Option<Entity>> {
    /// Performs a bevy query on both entities if present.
    ///
    /// # Panics
    /// Panics if the query cannot be used on the entities in `self`.
    #[must_use]
    pub fn query_mut<'a, D: QueryData, F: QueryFilter>(
        &self,
        query: &'a mut system::Query<D, F>,
    ) -> Binary<Option<QueryItem<'a, D>>> {
        let [alpha, beta] = match *self {
            Binary { alpha: Some(alpha), beta: Some(beta) } => {
                query.many_mut([alpha, beta]).map(Some)
            }
            Binary { alpha: Some(alpha), beta: None } => {
                [Some(query.get_mut(alpha).unwrap()), None]
            }
            Binary { alpha: None, beta: Some(beta) } => [None, Some(query.get_mut(beta).unwrap())],
            Binary { alpha: None, beta: None } => [None, None],
        };
        Binary { alpha, beta }
    }

    /// Performs a bevy query on both entities if present, returning the entity ID together.
    ///
    /// # Panics
    /// Panics if the query cannot be used on the entities in `self`.
    #[must_use]
    pub fn query_mut_with_entity<'a, D: QueryData, F: QueryFilter>(
        &self,
        query: &'a mut system::Query<D, F>,
    ) -> Binary<Option<(Entity, QueryItem<'a, D>)>> {
        let [alpha, beta] = match *self {
            Binary { alpha: Some(alpha), beta: Some(beta) } => {
                let [alpha_item, beta_item] = query.many_mut([alpha, beta]);
                [Some((alpha, alpha_item)), Some((beta, beta_item))]
            }
            Binary { alpha: Some(alpha), beta: None } => {
                [Some((alpha, query.get_mut(alpha).unwrap())), None]
            }
            Binary { alpha: None, beta: Some(beta) } => {
                [None, Some((beta, query.get_mut(beta).unwrap()))]
            }
            Binary { alpha: None, beta: None } => [None, None],
        };
        Binary { alpha, beta }
    }
}

/// The endpoint buildings of a corridor.
#[derive(Component)]
pub struct Endpoints {
    /// Endpoint buildings.
    pub endpoints: Binary<Entity>,
}

/// Length of a corridor.
///
/// Cached for reference, maintained by the corridor plugin and computed from endpoint positions.
#[derive(Default, Component)]
pub struct Length {
    /// Length value.
    pub value: f32,
}

/// List of ducts in a corridor.
#[derive(Component)]
pub struct DuctList {
    /// Non-ambient ducts in this corridor.
    /// The order of entities in this list has no significance.
    pub ducts: Vec<Entity>,

    /// The ambient duct for this corridor.
    pub ambient: Entity,
}
