//! An inhabitant represents a moving object that performs tasks.
//!
//! An inhabitant has the following characteristics:
//! - An inhabitant has skill values
//! - An inhabitant may be assigned to a node population storage, an edge or a vehicle.
//! - An inhabitant may contain cargo.
//! - An inhabitant may be assigned to a building with the Housing feature.

use serde::{Deserialize, Serialize};
use xylem::Xylem;

use crate::i18n::I18n;
use crate::{building, node, skill, unit};

/// Creates an inhabitant.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub struct Inhabitant {
    /// The copy-safe identifier.
    #[xylem(args(new = true))]
    pub id:     Id,
    /// The string identifier.
    #[xylem(serde(default))]
    pub id_str: IdString,

    /// Location of the inhabitant.
    pub location: Location,
    /// Housing assigned to the inhabitant.
    pub housing:  Option<node::Id>,
    /// Skill levels of the inhabitant.
    pub skills:   Vec<SkillLevel>,
}

impl_identifiable!(Inhabitant);

/// Sets the location of an inhabitant.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize), serde(tag = "type"))]
pub enum Location {
    Node(NodeLocation),
    // Edge(EdgeLocation),
    // Vehicle(VehicleLocation),
}

/// An inhabitant is currently in a node.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize), serde(tag = "type"))]
pub struct NodeLocation {
    /// The node that the inhabitant is currently located in.
    pub node:    node::Id,
    /// The population storage that holds the inhabitant currently.
    pub storage: StorageId,
}

/// Sets the skill level.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub struct SkillLevel {
    pub ty: skill::Id,
}

/// Defines a role of inhabitants and the number of inhabitants allowed.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub struct Storage {
    /// The copy-safe identifier.
    #[xylem(args(new = true))]
    pub id:      StorageId,
    /// The string identifier.
    #[xylem(serde(default))]
    pub id_str:  StorageIdString,
    /// The display name.
    pub name:    I18n,
    /// A short, one-line description.
    pub summary: I18n,
}

impl_identifiable!(@Storage Storage);
