//! Saving game definition and state.

use serde::{Deserialize, Serialize};

use crate::def::GameDefinition;
use crate::edge::save::Edge;
use crate::node::save::Node;
use crate::time::Instant;

/// The schema for a `.tflsav` file.
#[derive(Serialize, Deserialize)]
pub struct SaveFile {
    def: GameDefinition,
    state: GameState,
}

/// The state of the game.
#[derive(Serialize, Deserialize)]
pub struct GameState {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    clock: Instant,
}
