use std::collections::BTreeMap;
use std::error::Error;
use std::f64::consts::PI;
use std::time as walltime;

use traffloat_def::edge::UndirectedAlphaBeta;
use traffloat_def::node::NodeId;
use traffloat_types::space::Vector;
use traffloat_types::time::{Instant, Time};
use xias::Xias;

use crate::{edge, node, texture, Event, Server, StdMeshes};

pub struct State {
    pub nodes: BTreeMap<NodeId, node::Prepared>,
    pub edges: BTreeMap<UndirectedAlphaBeta, edge::Prepared>,
    pub clock: Clock,
    pub sun:   Sun,

    pub std_meshes:   StdMeshes,
    pub texture_pool: texture::Pool,

    picked: Option<PickTarget>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            nodes: BTreeMap::new(),
            edges: BTreeMap::new(),
            clock: Clock {
                epoch:              Instant::default(),
                epoch_wall:         walltime::Instant::now(),
                wall_time_per_tick: walltime::Duration::from_millis(50),
            },
            sun:   Sun {
                initial: Vector::new(0.0, 0.0, 1.0),
                quarter: Vector::new(0.0, 0.0, 0.25),
                period:  Time(3000),
            },

            std_meshes:   StdMeshes::compute(),
            texture_pool: texture::Pool::default(),

            picked: None,
        }
    }
}

impl State {
    pub fn handle_event(
        &mut self,
        event: Event,
        gl: &three_d::Context,
        server: &mut impl Server,
    ) -> Result<(), Box<dyn Error>> {
        match event {
            Event::AddNode(node) => {
                let id = node.id;
                let prepared =
                    node::Prepared::new(node, &self.std_meshes, &self.texture_pool, gl, server)?;

                self.nodes.insert(id, prepared);
            }
            Event::RemoveNode(node) => {
                self.nodes.remove(&node);
            }
            Event::AddEdge(edge) => {
                let endpoints = [
                    self.nodes
                        .get(&edge.id.alpha())
                        .expect("Server sent invalid event")
                        .view
                        .position,
                    self.nodes
                        .get(&edge.id.beta())
                        .expect("Server sent invalid event")
                        .view
                        .position,
                ];

                let id = UndirectedAlphaBeta(edge.id);
                let prepared = edge::Prepared::new(edge, &self.std_meshes.cylinder, gl, endpoints)?;

                self.edges.insert(id, prepared);
            }
            Event::RemoveEdge(edge) => {
                self.edges.remove(&UndirectedAlphaBeta(edge));
            }
            Event::SetClock(patch) => {
                self.clock = Clock {
                    epoch:              patch.now,
                    epoch_wall:         walltime::Instant::now(),
                    wall_time_per_tick: patch.wall_time_per_tick,
                };
            }
            Event::SetSun(sun) => {
                self.sun = sun;
            }
        }

        Ok(())
    }

    pub fn set_picked(&mut self, target: Option<PickTarget>) {
        for (target, is_picked) in [(self.picked.as_ref(), false), (target.as_ref(), true)] {
            match &target {
                Some(PickTarget::Node(id)) => {
                    if let Some(node) = self.nodes.get_mut(id) {
                        node.set_picked(is_picked);
                    }
                }
                Some(PickTarget::Edge(id)) => {
                    if let Some(edge) = self.edges.get_mut(id) {
                        edge.set_picked(is_picked);
                    }
                }
                None => {}
            }
        }

        self.picked = target;
    }
}

pub struct Clock {
    pub epoch:              Instant,
    pub epoch_wall:         walltime::Instant,
    pub wall_time_per_tick: walltime::Duration,
}

impl Clock {
    pub fn now(&self) -> Instant {
        let wall_elapsed = self.epoch_wall.elapsed();
        let ticks = wall_elapsed.div_duration_f64(self.wall_time_per_tick);
        self.epoch + Time(ticks.trunc_int())
    }
}

pub struct Sun {
    pub initial: Vector,
    pub quarter: Vector,
    pub period:  Time,
}

impl Sun {
    pub fn source_direction(&self, clock: &Clock) -> Vector {
        let time = (clock.now().since_epoch() % self.period).value().small_float::<f64>()
            / self.period.value().small_float::<f64>();
        let angle = PI * 2. * time;

        let (sin, cos) = angle.sin_cos();
        self.initial * cos + self.quarter * sin
    }
}

pub enum PickTarget {
    Node(NodeId),
    Edge(UndirectedAlphaBeta),
}
