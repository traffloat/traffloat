use std::collections::BTreeMap;
use std::f64::consts::PI;
use std::time as walltime;

use nalgebra::Rotation3;
use three_d::{CPUMaterial, CPUMesh, GeometryMut};
use traffloat_def::edge;
use traffloat_def::node::NodeId;
use traffloat_types::geometry;
use traffloat_types::space::{Matrix, Position, Vector};
use traffloat_types::time::{Instant, Time};
use typed_builder::TypedBuilder;
use xias::Xias;

use crate::{mat, Event};

#[derive(TypedBuilder)]
pub struct State {
    pub nodes: BTreeMap<NodeId, PreparedNodeView>,
    pub edges: BTreeMap<edge::UndirectedAlphaBeta, PreparedEdgeView>,
    pub clock: Clock,
    pub sun:   Sun,

    cylinder: CPUMesh,
}

impl Default for State {
    fn default() -> Self {
        Self {
            nodes:    BTreeMap::new(),
            edges:    BTreeMap::new(),
            clock:    Clock {
                epoch:              Instant::default(),
                epoch_wall:         walltime::Instant::now(),
                wall_time_per_tick: walltime::Duration::from_millis(50),
            },
            sun:      Sun {
                initial: Vector::new(0.0, 0.0, 1.0),
                quarter: Vector::new(0.0, 0.0, 0.25),
                period:  Time(3000),
            },
            cylinder: CPUMesh::cylinder(16),
        }
    }
}

impl State {
    pub fn handle_event(&mut self, event: Event, gl: &three_d::Context) {
        match event {
            Event::AddNode(node) => {
                self.nodes.insert(node.id, PreparedNodeView::new(node, &self.cylinder, gl));
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

                let id = edge::UndirectedAlphaBeta(edge.id);
                let prepared = PreparedEdgeView::new(edge, &self.cylinder, gl, endpoints);

                self.edges.insert(id, prepared);
            }
            Event::RemoveEdge(edge) => {
                self.edges.remove(&edge::UndirectedAlphaBeta(edge));
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
    }
}

pub struct PreparedNodeView {
    view:  NodeView,
    model: three_d::Model<three_d::ColorMaterial>,
}

impl PreparedNodeView {
    pub fn new(view: NodeView, cylinder: &CPUMesh, gl: &three_d::Context) -> Self {
        Self {
            view,
            model: three_d::Model::new_with_material(
                gl,
                cylinder,
                three_d::ColorMaterial::new(gl, &CPUMaterial::default()).unwrap(),
            )
            .unwrap(),
        }
    }

    pub fn object(&self) -> &dyn three_d::Object { &self.model }
}

pub struct NodeView {
    pub id:        NodeId,
    pub position:  Position,
    pub transform: Matrix,
    pub shape:     geometry::Unit,
}

pub struct PreparedEdgeView {
    view:  EdgeView,
    model: three_d::Model<three_d::PhysicalMaterial>,
}

impl PreparedEdgeView {
    pub fn new(
        view: EdgeView,
        cylinder: &CPUMesh,
        gl: &three_d::Context,
        endpoints: [Position; 2],
    ) -> Self {
        let mut this = Self {
            view,
            model: three_d::Model::new_with_material(
                gl,
                cylinder,
                three_d::PhysicalMaterial { ..Default::default() },
            )
            .unwrap(),
        };
        this.set_endpoints(endpoints[0], endpoints[1]);
        this
    }

    pub fn set_endpoints(&mut self, alpha: Position, beta: Position) {
        let diff = beta.value() - alpha.value();
        let mut tf = match Rotation3::rotation_between(&Vector::new(1., 0., 0.), &diff) {
            Some(rot) => rot.to_homogeneous(),
            None => Matrix::identity(),
        };
        tf.append_nonuniform_scaling_mut(&Vector::new(
            diff.norm(),
            self.view.radius,
            self.view.radius,
        ));
        tf.append_translation_mut(&alpha.vector());
        self.model.set_transformation(mat(tf));
    }

    pub fn object(&self) -> &dyn three_d::Object { &self.model }
}

pub struct EdgeView {
    pub id:     edge::AlphaBeta,
    pub radius: f64,
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
