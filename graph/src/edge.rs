use dynec::Entity;
use nalgebra::Vector3;

use crate::node::Node;

dynec::archetype! {
    /// An edge connects two different nodes.
    ///
    /// The two endpoint nodes are called "alpha" and "beta"
    /// instead of "source"/"destination" or "from"/"to"
    /// because this is ambiguous in conditions where a direction is reversed.
    pub Edge;
}

/// Identifies between the two endpoints of an edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, dynec::Discrim)]
pub enum WhichEndpoint {
    /// The first endpoint. This is not necessarily the starting point.
    Alpha,
    /// The second endpoint. This is not necessarily the ending point.
    Beta,
}

impl WhichEndpoint {
    pub fn peer(self) -> Self {
        match self {
            Self::Alpha => Self::Beta,
            Self::Beta => Self::Alpha,
        }
    }

    pub fn alpha_if(cond: bool) -> Self {
        if cond {
            Self::Alpha
        } else {
            Self::Beta
        }
    }

    pub fn beta_if(cond: bool) -> Self {
        if cond {
            Self::Beta
        } else {
            Self::Alpha
        }
    }
}

/// Utility class to retrieve values by endpoint.
#[derive(Debug, Clone, Copy)]
pub struct EndpointValues<T> {
    pub alpha: T,
    pub beta:  T,
}

impl<T> EndpointValues<T> {
    pub fn from_array([alpha, beta]: [T; 2]) -> Self { Self { alpha, beta } }

    /// Gets an endpoint value by refreence.
    pub fn get(self, endpoint: WhichEndpoint) -> T {
        match endpoint {
            WhichEndpoint::Alpha => self.alpha,
            WhichEndpoint::Beta => self.beta,
        }
    }

    /// Gets an endpoint value by refreence.
    pub fn get_ref(&self, endpoint: WhichEndpoint) -> &T {
        match endpoint {
            WhichEndpoint::Alpha => &self.alpha,
            WhichEndpoint::Beta => &self.beta,
        }
    }

    /// Gets an endpoint value by refreence.
    pub fn get_mut(&mut self, endpoint: WhichEndpoint) -> &mut T {
        match endpoint {
            WhichEndpoint::Alpha => &mut self.alpha,
            WhichEndpoint::Beta => &mut self.beta,
        }
    }

    pub fn bimap<'t, U: 't, R: 't>(
        &'t self,
        other: &'t EndpointValues<U>,
        mut map: impl FnMut(&'t T, &'t U) -> R,
    ) -> EndpointValues<R> {
        EndpointValues {
            alpha: map(&self.alpha, &other.alpha),
            beta:  map(&self.beta, &other.beta),
        }
    }
}

/// Stores the alpha and beta nodes of this edge.
///
/// When a node is destroyed, the edge may still remain,
/// in which case the `node` field would be removed.
#[dynec::comp(of = Edge, isotope = WhichEndpoint)]
pub struct Endpoint {
    #[entity]
    pub node: Entity<Node>,
}

/// The position of an edge endpoint.
///
/// The position of an edge endpoint is usually on the surface of the endpoint node.
#[dynec::comp(of = Edge, isotope = WhichEndpoint)]
pub struct EndpointPosition {
    pub position: Vector3<f64>,
}

/// The radius of an edge.
#[dynec::comp(of = Edge, required)]
pub struct Radius {
    pub quantity: f64,
}

/// The length of the edge, which should be in sync with the difference of its `EndpointPosition`.
#[dynec::comp(of = Edge, required)]
pub struct Length {
    pub quantity: f64,
}
