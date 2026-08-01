//! Relationship between archetype and key component types in the physics world..
//!
//! # Building
//! Components:
//! - [`graph::Building`]
//! - [`fluid::Storage`] (for ambient storage)
//! - [`view::Viewable`]
//!
//! Parent of:
//! - Facility.
//! - Graph Edge
//!
//! # Facility
//! Components:
//! - [`graph::Facility`]
//! - [`graph::FacilityType`]
//! - [`view::Viewable`]
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
//! - [`view::Viewable`]
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
//! - [`fluid::Edge`]
//! - [`graph::connection::Connection`]
//! - [`graph::connection::MainFacility`] (the "main" source facility)
//! - Depending on the peer type,
//!   - [`graph::connection::ToBuilding`], referencing the parent building entity
//!   - [`graph::connection::AltFacility`], referencing the peer facility entity
//!   - [`graph::connection::ToPipe`], referencing the adjacent conduit entity
//! - [`fluid::EdgeAlpha`], referencing the main facility
//! - [`fluid::EdgeBeta`], referencing the peer storage
//!
//! # Facility Type
//! Components:
//! - [`graph::FacilityTypeDef`]
//!
//! Parent of:
//! - Facility
//!
//! # Conduit
//! Components:
//! - [`graph::Conduit`]
//! - [`view::Viewable`]
//! - Fluid conduits:
//!   - [`fluid::Storage`]
//!   - [`fluid::Sensor`]
//! - Rails:
//!   - [`vehicle::Rail`]
//!
//! Child of:
//! - Corridor
//!
//! # Resident
//! Components:
//! - [`resident::Resident`]
//! - [`resident::Location`]
//! - [`resident::InteractingWith`], if interacting with a facility
//! - [`vehicle::PassengerOfCompartment`], if riding in a vehicle
//! - [`vehicle::OperatorOf`], if operating a vehicle
//! - [`view::Viewable`]
//!
//! # Vehicle
//! Components:
//! - [`vehicle::Vehicle`]
//! - [`vehicle::Location`]
//! - [`vehicle::OperatorList`]
//! - [`view::Viewable`]
//! - [`vehicle::propulsion::Desired`]
//! - [`vehicle::propulsion::Status`]
//!
//! Parent of:
//! - Vehicle compartment
//!
//! # Vehicle compartment
//! Components:
//! - [`vehicle::CompartmentOf`]
//! - [`vehicle::CompartmentPassengerList`]
//! - [`fluid::Storage`]
//! - [`fluid::Sensor`]
//!
//! Child of:
//! - Vehicle
//!
//! Parent of:
//! - Vehicle compartment vent
//!
//! # Vehicle compartment vent
//! - [`fluid::Edge`]
//! - [`fluid::EdgeAlpha`], referencing the compartment
//! - [`fluid::EdgeBeta`], referencing the building/corridor ambient storage
//!
//! Child of:
//! - Vehicle compartment
//!
//! Note: every time the vehicle moves to a new fixture,
//! the vent entities are despawned and recreated.
//!
//! # Viewer
//! Components:
//! - [`view::Viewer`]
//! - [`fluid::ViewerSynced`]

use crate::*;
