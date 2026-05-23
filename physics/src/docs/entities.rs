//! Relationship between archetype and key component types.
//!
//! # Building
//! Components:
//! - [`graph::Building`]
//! - [`fluid::Storage`] (for ambient storage)
//!
//! Parent of:
//! - Facility.
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

use crate::*;
