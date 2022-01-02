use std::time as walltime;

use anyhow::Result;
use traffloat_def::edge;
use traffloat_def::node::NodeId;
use traffloat_types::time::Instant;

use crate::{EdgeView, NodeView, Sun};

/// An abstraction of the simulation source.
pub trait Server: 'static {
    /// Receives an event from the simulation source.
    fn receive(&mut self) -> Result<Option<Event>>;
}

pub enum Event {
    AddNode(NodeView),
    RemoveNode(NodeId),
    AddEdge(EdgeView),
    RemoveEdge(edge::AlphaBeta),
    SetClock(SetClock),
    SetSun(Sun),
}

pub struct SetClock {
    pub now:                Instant,
    pub wall_time_per_tick: walltime::Duration,
}
