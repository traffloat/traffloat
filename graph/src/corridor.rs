//! Edges in the structural graph.

use std::ops;

use dynec::{entity, Entity};

use crate::building::Building;

dynec::archetype! {
    /// A link between two buildings
    pub Corridor;

    /// An internal structure of a corridor.
    pub Duct;
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
pub struct Binary<T> {
    /// The value for the alpha endpoint.
    pub alpha: T,
    /// The value for the beta endpoint.
    pub beta:  T,
}

impl<T> Binary<T> {
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

    /// Returns the value corresponding to first_endpoint and to `!first_endpoint` respectively.
    pub fn as_endpoints_mut(&mut self, first_endpoint: Endpoint) -> (&mut T, &mut T) {
        match first_endpoint {
            Endpoint::Alpha => (&mut self.alpha, &mut self.beta),
            Endpoint::Beta => (&mut self.beta, &mut self.alpha),
        }
    }
}

impl<T: entity::Referrer> entity::Referrer for Binary<T> {
    fn visit_type(arg: &mut dynec::entity::referrer::VisitTypeArg) {
        if arg.mark::<Self>().is_continue() {
            T::visit_type(arg);
        }
    }

    fn visit_mut<V: dynec::entity::referrer::VisitMutArg>(&mut self, arg: &mut V) {
        self.alpha.visit_mut(arg);
        self.beta.visit_mut(arg);
    }
}

/// The endpoint buildings of a corridor.
#[dynec::comp(of = Corridor, required)]
pub struct Endpoints {
    #[entity]
    pub endpoints: Binary<Entity<Building>>,
}

/// List of ducts in a corridor.
#[dynec::comp(of = Corridor, required)]
pub struct DuctList {
    /// Non-ambient ducts in this corridor.
    /// The order of entities in this list has no significance.
    #[entity]
    pub ducts: Vec<Entity<Duct>>,

    /// The ambient duct for this corridor.
    #[entity]
    pub ambient: Entity<Duct>,
}

/// References the owning building for a facility.
#[dynec::comp(of = Duct, required)]
pub struct DuctOwner {
    /// The corridor in which this duct is installed.
    #[entity]
    pub corridor: Entity<Corridor>,
}
