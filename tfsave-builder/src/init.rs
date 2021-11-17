use serde::Deserialize;
use traffloat_def::{self as def, State};
use traffloat_types::time;

#[derive(Clone, Deserialize)]
pub struct ScalarState {
    pub time: time::Instant,
}

#[derive(xylem::Xylem)]
#[xylem(schema = def::Schema, expose = InitXylem, derive(Deserialize), serde(tag = "type"))]
pub enum Init {
    Node(Box<def::node::Node>),
    Edge(Box<def::edge::Edge>),
}

pub fn resolve_states(inits: impl IntoIterator<Item = Init>, scalar: &ScalarState) -> State {
    let mut state = State::new(scalar.time);

    for init in inits {
        match init {
            Init::Node(node) => state.nodes_mut().push(*node),
            Init::Edge(edge) => state.edges_mut().push(*edge),
        }
    }

    state
}
