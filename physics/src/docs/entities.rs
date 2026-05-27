//! Relationship between archetype and key component types.
//!
//! # Building
//! Components:
//! - [`graph::Building`]
//! - [`fluid::Storage`] (for ambient storage)
//!
//! Parent of:
//! - Facility.
//! - Graph Edge
//!
//! # Facility
//! Components:
//! - [`graph::Facility`]
//! - [`graph::FacilityType`]
//! - By facility type:
//!   - [`fluid::Storage`]
//!   - [`reactor::Facility`]
//!
//! Child of:
//! - Building.
//! - Facility Type
//!
//! # Corridor
//! Components:
//! - [`graph::Corridor`]
//! - [`fluid::Storage`] (for ambient storage)
//!
//! Parent of:
//! - Conduit.
//! - Graph Edge
//!
//! # Graph edge
//! Describes the connection between a corridor and a building.
//!
//! Components:
//! - [`graph::edge::Edge`]
//! - [`graph::edge::OfBuilding`], [`graph::edge::OfCorridor`]
//! - [`fluid::Edge`] (when open)
//!     - [`fluid::EdgeAlpha`] always points to the building side of the edge.
//!     - [`fluid::EdgeBeta`] always points to the corridor side of the edge.
//!
//! # Conduit
//! Components:
//! - [`graph::Conduit`]
//! - Fluid conduits:
//!   - [`fluid::Storage`]
//!
//! Child of:
//! - Corridor
//!
//! # Facility Type
//! Parent of:
//! - Facility
//!
//! # Fluid edges
//! Components:
//! - [`fluid::Edge`]
//! - [`fluid::EdgeAlpha`]
//! - [`fluid::EdgeBeta`]
//!
//! # Viewer
//! Components:
//! - [`view::Viewer`]
//! - [`fluid::ViewerSynced`]

use crate::*;
