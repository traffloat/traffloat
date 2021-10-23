//! Defines the starting state of a game.
//!
//! Data structures in this module do not duplicate information from the scenario,
//! except [`CustomizableName`], which is necessary e.g. in case of building upgrades.

use getset::{CopyGetters, Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};
use traffloat_types::time;

pub mod appearance;

mod node;
pub use node::*;

mod edge;
pub use edge::*;

/// The state of objects in a game.
#[derive(Default, Getters, Setters, CopyGetters, MutGetters, Serialize, Deserialize)]
pub struct State {
    /// Current game time.
    #[getset(get_copy = "pub")]
    #[getset(set = "pub")]
    time:  time::Instant,
    /// State of all nodes in the game.
    #[getset(get = "pub")]
    #[getset(get_mut = "pub")]
    nodes: Vec<Node>,
    /// State of all edges in the game.
    #[getset(get = "pub")]
    #[getset(get_mut = "pub")]
    edges: Vec<Edge>,
}
