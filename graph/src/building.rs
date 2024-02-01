//! Nodes in the structural graph.

use dynec::Entity;
use nalgebra::Vector3;

dynec::archetype! {
    /// A building in which facilities can be installed.
    pub Building;

    /// An internal structure of a building.
    pub Facility;
}

/// Reference position of a building.
#[dynec::comp(of = Building, required)]
pub struct Position {
    pub position: Vector3<f64>,
}

/// List of facilities in a building.
#[dynec::comp(of = Building, required)]
pub struct FacilityList {
    /// Non-ambient facilities in this building.
    /// The order of entities in this list has no significance.
    #[entity]
    pub facilities: Vec<Entity<Facility>>,

    /// The ambient space for this building.
    #[entity]
    pub ambient: Entity<Facility>,
}

/// References the owning building for a facility.
#[dynec::comp(of = Facility, required)]
pub struct FacilityOwner {
    /// The building in which this facility is installed.
    #[entity]
    pub building: Entity<Building>,
}
