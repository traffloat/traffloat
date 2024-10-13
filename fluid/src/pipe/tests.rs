use std::iter;

use approx::assert_relative_eq;
use bevy::app::App;
use bevy::ecs::world::Command;
use bevy::state::app::{AppExtStates, StatesPlugin};
use bevy::time::TimePlugin;
use traffloat_base::{save, EmptyState};
use traffloat_graph::corridor::Binary;
use typed_builder::TypedBuilder;

use crate::config::{self, Scalar};
use crate::{commands, container, pipe, units};

struct Setup {
    elements:   Vec<ElementSetup>,
    containers: Binary<ContainerSetup>,
}

#[derive(TypedBuilder)]
struct ElementSetup {
    #[builder(setter(into))]
    viscosity:              units::Viscosity,
    #[builder(setter(into))]
    vacuum_specific_volume: units::SpecificVolume,
    #[builder(setter(into))]
    critical_pressure:      units::Pressure,
    saturation_gamma:       f32,
    #[builder(setter(transform = |[alpha, beta]: [f32; 2]| [alpha.into(), beta.into()].into()))]
    mass:                   Binary<units::Mass>,
}

#[derive(TypedBuilder)]
struct ContainerSetup {
    #[builder(setter(into))]
    max_pressure: units::Pressure,
    #[builder(setter(into))]
    max_volume:   units::Volume,
}

fn do_test(setup: Setup) {
    let mut app = App::new();
    app.add_plugins((
        TimePlugin,
        StatesPlugin,
        save::Plugin,
        traffloat_view::Plugin,
        config::Plugin,
        container::Plugin(EmptyState),
        pipe::Plugin(EmptyState),
    ));
    app.init_state::<EmptyState>();

    let config = Scalar::default();
    let types: Vec<_> = setup
        .elements
        .iter()
        .map(|element| {
            config::create_type(
                &mut app.world_mut().commands(),
                config::TypeDef {
                    viscosity:              element.viscosity,
                    vacuum_specific_volume: element.vacuum_specific_volume,
                    critical_pressure:      element.critical_pressure,
                    saturation_gamma:       element.saturation_gamma,
                },
            )
        })
        .collect();
    app.insert_resource(config);

    let containers = Binary::from_fn(|endpoint| {
        let container_setup = setup.containers.as_endpoint(endpoint);
        let entity = app.world_mut().spawn(
            container::Bundle::builder()
                .max_volume(container_setup.max_volume)
                .max_pressure(container_setup.max_pressure)
                .build(),
        );
        let entity = entity.id();

        for (element, &ty) in iter::zip(&setup.elements, &types) {
            commands::CreateContainerElement::builder()
                .container(entity)
                .ty(ty)
                .mass(element.mass.into_endpoint(endpoint))
                .build()
                .apply(app.world_mut());
        }

        entity
    });

    let _pipe = {
        let entity = app.world_mut().spawn(
            pipe::Bundle::builder()
                .shape_resistance(units::Resistance { quantity: 1. })
                .containers(containers)
                .build(),
        );
        entity.id()
    };

    for _ in 0..100 {
        app.update();
    }

    // Assert that the pressure of the containers will reach equilibrium.
    let pressure = containers.map(|container| {
        app.world().get::<container::CurrentPressure>(container).unwrap().pressure.quantity
    });
    assert_relative_eq!(pressure.alpha, pressure.beta);
}

#[test]
fn empty_containers() {
    do_test(Setup {
        elements:   vec![],
        containers: [
            ContainerSetup::builder().max_pressure(10.).max_volume(10.).build(),
            ContainerSetup::builder().max_pressure(10.).max_volume(10.).build(),
        ]
        .into(),
    });
}

#[test]
fn filled_to_empty() {
    do_test(Setup {
        elements:   vec![ElementSetup::builder()
            .viscosity(1.)
            .vacuum_specific_volume(1.)
            .critical_pressure(10.)
            .saturation_gamma(10.)
            .mass([1., 0.])
            .build()],
        containers: [
            ContainerSetup::builder().max_pressure(10.).max_volume(10.).build(),
            ContainerSetup::builder().max_pressure(10.).max_volume(10.).build(),
        ]
        .into(),
    });
}
