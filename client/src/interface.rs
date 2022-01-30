use std::time as walltime;

use anyhow::Result;
use traffloat_def::edge::AlphaBeta;
use traffloat_def::node::NodeId;
use traffloat_types::time::Instant;

use crate::{edge, node, Sun};

/// An abstraction of the simulation source.
pub trait Server: 'static {
    /// Receives an event from the simulation source.
    fn receive(&mut self) -> Result<Option<Event>>;

    /// Resolves the path to load an image asset file.
    fn load_asset(&self, name: &str) -> String;
}

pub enum Event {
    AddNode(node::View),
    RemoveNode(NodeId),
    AddEdge(edge::View),
    RemoveEdge(AlphaBeta),
    SetClock(SetClock),
    SetSun(Sun),
}

pub struct SetClock {
    pub now:                Instant,
    pub wall_time_per_tick: walltime::Duration,
}
