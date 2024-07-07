use std::iter;

use approx::assert_relative_eq;
use bevy::app::App;
use bevy::ecs::world::Command;
use bevy::hierarchy;
use traffloat_graph::corridor::Binary;
use typed_builder::TypedBuilder;

use crate::{commands, config, container, pipe, units};

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
    app.add_plugins((container::Plugin, pipe::Plugin));

    let mut builder = config::ConfigBuilder::default();
    let types: Vec<_> = setup
        .elements
        .iter()
        .map(|element| {
            builder.register_type(config::TypeDef {
                viscosity:              element.viscosity,
                vacuum_specific_volume: element.vacuum_specific_volume,
                critical_pressure:      element.critical_pressure,
                saturation_gamma:       element.saturation_gamma,
            })
        })
        .collect();
    app.insert_resource(builder.build());

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

    let pipe = {
        let entity = app
            .world_mut()
            .spawn(pipe::Bundle::builder().shape_resistance(1.).containers(containers).build());
        entity.id()
    };

    for _ in 0..100 {
        app.update();
        let pressure = containers.map(|container| {
            app.world().get::<container::CurrentPressure>(container).unwrap().pressure.quantity
        });
        dbg!(pressure);

        let children = app.world().get::<hierarchy::Children>(pipe);
        for &child in children.into_iter().flatten() {
            let ty = app.world().get::<config::Type>(child);
            let transfer = app.world().get::<pipe::element::AbTransferMass>(child);
            dbg!(ty, transfer.map(|t| t.mass));
        }
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
