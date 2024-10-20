use std::iter;

use approx::assert_relative_eq;
use bevy::app::App;
use bevy::hierarchy::BuildWorldChildren;
use bevy::state::app::{AppExtStates, StatesPlugin};
use bevy::time::TimePlugin;
use traffloat_base::{save, EmptyState};
use traffloat_view::DisplayText;

use super::element;
use crate::config::{self, Scalar};
use crate::units;

struct ContainerSetup {
    max_pressure:    f32,
    max_volume:      f32,
    expect_pressure: f32,
    elements:        Vec<ElementSetup>,
}

struct ElementSetup {
    mass:                   f32,
    vacuum_specific_volume: f32,
    critical_pressure:      f32,
    saturation_gamma:       f32,
    expect_volume:          f32,
}

fn do_test(setup: ContainerSetup) {
    let mut app = App::new();
    app.add_plugins((
        TimePlugin,
        StatesPlugin,
        save::Plugin,
        traffloat_view::Plugin,
        config::Plugin,
    ));
    app.init_state::<EmptyState>();

    let types: Vec<_> = setup
        .elements
        .iter()
        .map(|fluid| {
            config::create_type(
                &mut app.world_mut().commands(),
                config::TypeDef {
                    display_label:          DisplayText::default(),
                    viscosity:              units::Viscosity::default(), // unused
                    vacuum_specific_volume: fluid.vacuum_specific_volume.into(),
                    critical_pressure:      fluid.critical_pressure.into(),
                    saturation_gamma:       fluid.saturation_gamma,
                },
            )
        })
        .collect();

    let config = Scalar::default();
    app.insert_resource(config);
    app.add_plugins(super::Plugin(EmptyState));

    let mut container = app.world_mut().spawn(
        super::Bundle::builder()
            .max_volume(super::MaxVolume { volume: setup.max_volume.into() })
            .max_pressure(super::MaxPressure { pressure: setup.max_pressure.into() })
            .build(),
    );

    let mut element_entities = Vec::new();
    container.with_children(|builder| {
        for (&ty, element) in iter::zip(&types, &setup.elements) {
            element_entities.push(
                builder
                    .spawn(
                        element::Bundle::builder()
                            .ty(ty)
                            .mass(element::Mass { mass: element.mass.into() })
                            .build(),
                    )
                    .id(),
            );
        }
    });

    let container_entity = container.id();

    app.update();

    assert_relative_eq!(
        app.world().get::<super::CurrentVolume>(container_entity).unwrap().volume.quantity,
        setup.elements.iter().map(|fluid| fluid.expect_volume).sum(),
    );
    assert_relative_eq!(
        app.world().get::<super::CurrentPressure>(container_entity).unwrap().pressure.quantity,
        setup.expect_pressure,
    );

    for (element, element_entity) in iter::zip(&setup.elements, element_entities) {
        assert_relative_eq!(
            app.world().get::<element::Volume>(element_entity).unwrap().volume.quantity,
            element.expect_volume,
        );
    }
}

#[test]
fn empty_container() {
    do_test(ContainerSetup {
        max_pressure:    100.,
        max_volume:      100.,
        expect_pressure: 0.,
        elements:        vec![],
    });
}

#[test]
fn mixture_vacuum() {
    do_test(ContainerSetup {
        max_pressure:    100.,
        max_volume:      100.,
        expect_pressure: (10. + 6.) / 100.,
        elements:        vec![
            ElementSetup {
                mass:                   5.,
                vacuum_specific_volume: 2.,
                critical_pressure:      50.,
                saturation_gamma:       100.,
                expect_volume:          10.,
            },
            ElementSetup {
                mass:                   2.,
                vacuum_specific_volume: 3.,
                critical_pressure:      50.,
                saturation_gamma:       100.,
                expect_volume:          6.,
            },
        ],
    });
}

#[test]
fn mixture_compression() {
    do_test(ContainerSetup {
        max_pressure:    100.,
        max_volume:      100.,
        expect_pressure: (72. + 60.) / 100.,
        elements:        vec![
            ElementSetup {
                mass:                   9.,
                vacuum_specific_volume: 8.,
                critical_pressure:      50.,
                saturation_gamma:       100.,
                expect_volume:          72. / (72. + 60.) * 100.,
            },
            ElementSetup {
                mass:                   30.,
                vacuum_specific_volume: 2.,
                critical_pressure:      50.,
                saturation_gamma:       100.,
                expect_volume:          60. / (72. + 60.) * 100.,
            },
        ],
    });
}

#[test]
fn mixture_saturation() {
    do_test(ContainerSetup {
        max_pressure:    100.,
        max_volume:      100.,
        expect_pressure: 200. / 100. + (80. / (80. + 120.)) * (2. - 1.2) * 10.,
        elements:        vec![
            ElementSetup {
                mass:                   80.,
                vacuum_specific_volume: 1.,
                critical_pressure:      1.2,
                saturation_gamma:       10.,
                expect_volume:          80. / (80. + 120.) * 100.,
            },
            ElementSetup {
                mass:                   60.,
                vacuum_specific_volume: 2.,
                critical_pressure:      100.,
                saturation_gamma:       100.,
                expect_volume:          120. / (80. + 120.) * 100.,
            },
        ],
    });
}
