//! Relationship between archetype and key component types in the physics world..
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
//! - [`resident::InteractionSlots`]
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
//! # Facility connection
//! Describes the connection from a facility to one of the following:
//! - Its parent building (ambient fluid)
//! - Another facility in the same building
//! - A fluid conduit in an adjacent corridor
//!
//! Components:
//! - [`graph::connection::Connection`]
//! - [`graph::connection::MainFacility`] (the "main" source facility)
//! - Depending on the peer type,
//!   - [`graph::connection::ToBuilding`], referencing the parent building entity
//!   - [`graph::connection::AltFacility`], referencing the peer facility entity
//!   - [`graph::connection::ToPipe`], referencing the adjacent conduit entity
//!
//! # Conduit
//! Components:
//! - [`graph::Conduit`]
//! - Fluid conduits:
//!   - [`fluid::Storage`]
//!   - [`fluid::Sensor`]
//!
//! Child of:
//! - Corridor
//!
//! # Facility Type
//! Components:
//! - [`graph::FacilityTypeDef`]
//!
//! Parent of:
//! - Facility
//!
//! # Fluid edges
//! Components:
//! - [`fluid::Edge`]
//! - [`fluid::EdgeAlpha`]
//! - [`fluid::EdgeBeta`]
//!
//! # Resident
//! Components:
//! - [`resident::Resident`]
//! - [`resident::Location`]
//! - [`resident::InteractingWith`], if interacting with a facility
//!
//! # Viewer
//! Components:
//! - [`view::Viewer`]
//! - [`fluid::ViewerSynced`]

use crate::*;
