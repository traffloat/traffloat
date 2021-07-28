use traffloat::def::GameDefinition;
use traffloat::space::{Matrix, Position, Vector};
use traffloat::units;
use traffloat::SetupEcs;
use traffloat::{edge, node};

/// The default ECS setup
pub fn default_setup(mut setup: SetupEcs, def: &GameDefinition, ids: &super::AllIds) -> SetupEcs {
    let core = node::create_components(
        &mut setup.world,
        def,
        ids.building.core,
        Position::new(1., 2., 3.),
        Matrix::identity(),
    );
    let hut = node::create_components(
        &mut setup.world,
        def,
        ids.building.hut,
        Position::new(1., -2., 3.),
        Matrix::new_scaling(0.4),
    );
    let solar_panel = node::create_components(
        &mut setup.world,
        def,
        ids.building.solar_panel,
        Position::new(-2., 0., 10.),
        Matrix::new_nonuniform_scaling(&Vector::new(0.1, 0.5, 1.5)),
    );

    edge::create_components(
        &mut setup.world,
        core,
        hut,
        0.2,
        units::Portion::full(units::Hitpoint(100.)),
        vec![
            edge::save::SavedDuct {
                center: edge::CrossSectionPosition::new(0.1, 0.05),
                radius: 0.03,
                ty: edge::DuctType::Electricity(false),
            },
            edge::save::SavedDuct {
                center: edge::CrossSectionPosition::new(-0.12, -0.03),
                radius: 0.05,
                ty: edge::DuctType::Liquid(Some(edge::Direction::ToFrom)),
            },
            edge::save::SavedDuct {
                center: edge::CrossSectionPosition::new(0.07, -0.12),
                radius: 0.03,
                ty: edge::DuctType::Rail(Some(edge::Direction::FromTo)),
            },
        ],
    );
    edge::create_components(
        &mut setup.world,
        core,
        solar_panel,
        0.1,
        units::Portion::full(units::Hitpoint(100.)),
        Vec::new(),
    );

    setup
}
