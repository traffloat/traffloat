use indvec::indvec;
use legion::Entity;

use traffloat::def::GameDefinition;
use traffloat::{edge, node};
use traffloat_types::space::{Matrix, Position, Vector};

pub(crate) fn default_setup(
    def: &GameDefinition,
    building: &super::building::Ids,
) -> (Vec<node::Components>, EdgeSetup) {
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

    let edges = move |entities: &[Entity]| {
        vec![
            edge::create_components(
                entities[core],
                entities[hut],
                0.2,
                vec![
                    edge::Duct::builder()
                        .center(edge::CrossSectionPosition::new(0.1, 0.05))
                        .radius(0.03)
                        .ty(edge::DuctType::Electricity(false))
                        .entity(entities[0]) // TODO fix this
                        .build(),
                    edge::Duct::builder()
                        .center(edge::CrossSectionPosition::new(-0.12, -0.03))
                        .radius(0.05)
                        .ty(edge::DuctType::Liquid(Some(edge::Direction::ToFrom)))
                        .entity(entities[0]) // TODO fix this
                        .build(),
                    edge::Duct::builder()
                        .center(edge::CrossSectionPosition::new(0.07, -0.12))
                        .radius(0.03)
                        .ty(edge::DuctType::Rail(Some(edge::Direction::FromTo)))
                        .entity(entities[0]) // TODO fix this
                        .build(),
                ],
            ),
            edge::create_components(entities[core], entities[solar_panel], 0.1, vec![]),
        ]
    };

    (nodes, Box::new(edges))
}

pub type EdgeSetup = Box<dyn FnOnce(&[Entity]) -> Vec<edge::Components>>;
