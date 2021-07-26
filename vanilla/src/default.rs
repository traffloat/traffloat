use indvec::indvec;

use traffloat::def::GameDefinition;
use traffloat::node;
use traffloat_types::space::{Matrix, Position, Vector};

pub fn default_setup(
    def: &GameDefinition,
    building: &super::building::Ids,
) -> (Vec<node::Components>, Vec<(usize, usize, f64)>) {
    indvec![
        nodes = core = node::create_components(
            def,
            building.core,
            Position::new(1., 2., 3.),
            Matrix::identity(),
        ),
        hut = node::create_components(
            def,
            building.hut,
            Position::new(1., -2., 3.),
            Matrix::new_scaling(0.4),
        ),
        solar_panel = node::create_components(
            def,
            building.solar_panel,
            Position::new(-2., 0., 10.),
            Matrix::new_nonuniform_scaling(&Vector::new(0.1, 0.5, 1.5)),
        ),
    ];

    let edges = vec![(core, hut, 0.2), (core, solar_panel, 0.1)];

    (nodes, edges)
}
