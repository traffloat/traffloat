use anyhow::Result;
use traffloat_client::{Config, Event, NodeView, Server};
use traffloat_def::node::NodeId;
use traffloat_types::geometry;
use traffloat_types::space::{Matrix, Position};

fn main() {
    let mock = Mock::new(vec![Event::AddNode(NodeView {
        id:        NodeId::new(0),
        position:  Position::new(0., 0., 0.),
        transform: Matrix::identity(),
        shape:     geometry::Unit::Cylinder,
    })]);

    let config = Config::default();

    traffloat_client::run(mock, config).unwrap();
}

struct Mock {
    vec: std::vec::IntoIter<Event>,
}

impl Mock {
    fn new(vec: Vec<Event>) -> Self { Self { vec: vec.into_iter() } }
}

impl Server for Mock {
    fn receive(&mut self) -> Result<Option<Event>> { Ok(self.vec.next()) }
}
