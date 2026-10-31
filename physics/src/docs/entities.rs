//! This page outlines the relationship between archetype and key component types in the physics world.
//!
//! All objects without a parent should have the [`WorldObject`] component.
//!
//! # Building
//! Components:
//! - [`graph::Building`]
//! - Viewable
//! - [`fluid::Storage`] (for ambient storage)
//!     - [`fluid::Sensor`]
//! - [`cargo::Storage`]
//!
//! Parent of:
//! - Facility, via [`graph::facility::ListInBuilding`]
//! - Graph Edge, via [`graph::edge::BuildingEdges`]
//!
//! Non-cascading parent of:
//! - Resident, via [`resident::ListInAmbientFixture`]
//! - Vehicle, via [`vehicle::ListInBuilding`]
//!
//! # Facility
//! Components:
//! - [`graph::Facility`]
//! - Viewable
//! - By facility type:
//!   - [`fluid::Storage`]
//!     - [`fluid::Sensor`]
//!   - [`cargo::Storage`]
//!   - [`reactor::Facility`]
//!     - [`reactor::FacilityStatus`]
//! - [`resident::InteractionSlots`]
//! - [`resident::InteractingResidents`]
//!
//! Child of:
//! - Building, via [`graph::facility::OfBuilding`]
//! - Facility Type, via [`graph::FacilityType`]
//! - Facility Connection, via [`graph::connection::ListOnMainFacility`]
//!
//! # Corridor
//! Components:
//! - [`graph::Corridor`]
//! - Viewable
//! - [`fluid::Storage`] (for ambient storage)
//!     - [`fluid::Sensor`]
//! - [`cargo::Storage`]
//!
//! Parent of:
//! - Conduit, via [`graph::conduit::ListInCorridor`]
//! - Graph Edge, via [`graph::edge::CorridorEdge`]
//!
//! Non-cascading parent of:
//! - Resident, via [`resident::ListInAmbient`]
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
//! - Facility, via [`graph::FacilityTypeInstances`]
//!
//! # Conduit
//! Components:
//! - [`graph::Conduit`]
//! - Viewable
//! - Fluid conduits:
//!   - [`fluid::Storage`]
//!     - [`fluid::Sensor`]
//!   - [`fluid::Sensor`]
//! - Rails:
//!   - [`vehicle::Rail`]
//!   - [`vehicle::rail::Reservation`]
//!   - [`vehicle::ListOnRail`]
//!
//! Child of:
//! - Corridor, via [`graph::conduit::OfCorridor`]
//!
//! # Resident
//! Components:
//! - [`resident::Resident`]
//! - Viewable
//! - [`resident::Location`]
//! - [`resident::Attributes`]
//!   - `resident::LastSentAttributes`
//!
//! Non-cascading child of:
//! - Building, via [`resident::IsInAmbientFixture`]
//! - Corridor, via [`resident::IsInAmbientFixture`]
//! - Facility, via [`resident::InteractingWith`]
//! - Vehicle Copmartment, via [`vehicle::PassengerOfCompartment`]
//! - Vehicle, via [`vehicle::OperatorOf`], if operating a vehicle
//!
//! # Vehicle
//! Components:
//! - [`vehicle::Vehicle`]
//! - Viewable
//! - [`vehicle::Location`]
//! - [`vehicle::propulsion::Desired`]
//!   - [`vehicle::propulsion::Status`]
//! - [`vehicle::motion::Intent`]
//!
//! Parent of:
//! - Vehicle Compartment, via [`vehicle::CompartmentList`]
//!
//! Non-cascading parent of:
//! - [`vehicle::OperatorList`]
//!
//! # Vehicle Compartment
//! Components:
//! - [`fluid::Storage`]
//!     - [`fluid::Sensor`]
//! - [`cargo::Storage`]
//! - [`fluid::Sensor`]
//!
//! Child of:
//! - Vehicle, via [`vehicle::CompartmentOf`]
//!
//! Parent of:
//! - Vehicle compartment vent, via [`vehicle::CompartmentHasVent`]
//!
//! Non-cascading parent of:
//! - [`vehicle::CompartmentPassengerList`]
//!
//! # Vehicle compartment vent
//! - [`fluid::Edge`]
//! - [`fluid::EdgeAlpha`], referencing the compartment
//! - [`fluid::EdgeBeta`], referencing the building/corridor ambient storage
//!
//! Child of:
//! - Vehicle compartment, via [`vehicle::CompartmentVentOf`]
//!
//! Note: every time the vehicle moves to a new fixture,
//! the vent entities are despawned and recreated.
//!
//! # Viewer
//! Components:
//! - [`view::Viewer`]
//! - [`fluid::ViewerSynced`]
//! - [`cargo::ViewerSynced`]
//!
//! # Viewable
//! Viewables archetypes listed above have the following components:
//! - [`view::Viewable`]
//! - [`view::Named`]
//! - [`view::CullingRect`]

use crate::*;
