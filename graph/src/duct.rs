use dynec::Entity;
use nalgebra::Vector2;

use crate::edge::Edge;

/// The list of ducts inside the edge.
#[dynec::comp(of = Edge, required, init = Ducts::default/0)]
#[derive(Default)]
pub struct Ducts(#[entity] pub Vec<Entity<Duct>>);

dynec::archetype! {
    /// Stores the generic metadata of a duct.
    ///
    /// Specific variants of ducts should create a new archetype that contains their data,
    /// then create a component in this duct that references the specific entity.
    pub Duct;
}

/// The corresponding edge entity that owns this duct.
#[dynec::comp(of = Duct, required)]
pub struct Owner(#[entity] pub Entity<Edge>);

/// The radius of the duct.
#[dynec::comp(of = Duct, required)]
pub struct Radius(pub f64);

/// The position of the duct inside the edge.
///
/// In the current design, the relative position of a duct inside an edge
/// has no effect on semantics other than display,
/// so the interpretation of this position is up to the client,
/// but the cross product of (0, 1) &times; (1, 0) should point from alpha to beta.
#[dynec::comp(of = Duct, required)]
pub struct Position(pub Vector2<f64>);
