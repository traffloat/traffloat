use std::cmp::Ordering;
use std::time::Duration;
use std::{fmt, iter};

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::time;

use crate::{fluid, view};

const NUM_TYPES: usize = 16;

fn default_types() -> Vec<fluid::TypeDef> {
    iter::repeat_with(|| fluid::TypeDef {
        name:                 "".into(),
        molar_heat_capacity:  1.0,
        advective_fluidity:   1.0,
        diffusive_fluidity:   1.0,
        molar_density:        1.0,
        thermal_conductivity: 1e-4,
        optical_extinction:   [0.0; 3],
    })
    .take(NUM_TYPES)
    .collect()
}

#[test]
fn test_empty() {
    do_test(
        |_| {},
        fluid::Storage::vacuum(NUM_TYPES, 100.0, 1.5),
        fluid::Storage::vacuum(NUM_TYPES, 100.0, 1.5),
        fluid::Edge::new(NUM_TYPES, 1.0, 10.0),
    )
    .validate(20, |alpha, beta, edge| {
        expect_float(alpha.pressure, 0.0);
        expect_float(beta.pressure, 0.0);

        expect_float(alpha.mass, 0.0);
        expect_float(beta.mass, 0.0);

        expect_float(edge.last_heat.0, 0.0);
    });
}

#[test]
fn test_equilibrium_big_small() {
    do_test(
        |_| {},
        fluid::Storage::vacuum(NUM_TYPES, 100.0, 1.5)
            .with_heat(fluid::Energy(30000.0))
            .with_fluid(fluid::TypeId(0), 100.0),
        fluid::Storage::vacuum(NUM_TYPES, 10.0, 0.15)
            .with_heat(fluid::Energy(3000.0))
            .with_fluid(fluid::TypeId(0), 10.0),
        fluid::Edge::new(NUM_TYPES, 1.0, 10.0),
    )
    .validate(20, |alpha, beta, edge| {
        expect_float(alpha.pressure, 2.34375);
        expect_float(beta.pressure, 2.34375);

        expect_float(alpha.mass, 100.0);
        expect_float(beta.mass, 10.0);

        expect_float(edge.last_heat.0, 0.0);
    });
}

#[test]
fn test_diffusion_big_small() {
    do_test(
        |_| {},
        fluid::Storage::vacuum(NUM_TYPES, 100.0, 1.5)
            .with_heat(fluid::Energy(30000.0))
            .with_fluid(fluid::TypeId(0), 100.0),
        fluid::Storage::vacuum(NUM_TYPES, 10.0, 0.15)
            .with_heat(fluid::Energy(3000.0))
            .with_fluid(fluid::TypeId(1), 10.0),
        fluid::Edge::new(NUM_TYPES, 1.0, 10.0),
    )
    .validate(320, |alpha, beta, edge| {
        expect_float(alpha.pressure, 2.34375);
        expect_float(beta.pressure, 2.34375);

        expect_float(alpha.temperature, 300.0);
        expect_float(beta.temperature, 300.0);

        expect_float(alpha.mass, 100.0);
        expect_float(beta.mass, 10.0);

        expect_float_near(edge.last_heat.0, 0.0, 1e-3);

        for ty in 0..2 {
            expect_float_near(alpha.types[0].proportion - beta.types[0].proportion, 0.0, 1e-3);
        }

        for transfer in &edge.last_typed_transfer {
            expect_float_near(transfer.atob_transfer.0, 0.0, 1e-3);
        }
    });
}

#[test]
fn test_temperature_induced_advection() {
    // This test involves two stages:
    // 1. Initially, storage alpha is much hotter and hence has much higher pressure,
    //    so a strong decompression force carries hot fluids from alpha to beta.
    //    This carries mass and heat from alpha to beta.
    // 2. Alpha and beta exchange heat by convection and conduction.
    //    Alpha cools down and shrinks, while beta heats up and expands,
    //    but container volumes remain unchanged,
    //    so a pressure gradient is created that pushes fluids back from beta to alpha.
    do_test(
        |_| {},
        fluid::Storage::vacuum(NUM_TYPES, 100.0, 1.5)
            .with_heat(fluid::Energy(40000.0))
            .with_fluid(fluid::TypeId(0), 100.0),
        fluid::Storage::vacuum(NUM_TYPES, 100.0, 1.5)
            .with_heat(fluid::Energy(20000.0))
            .with_fluid(fluid::TypeId(0), 100.0),
        fluid::Edge::new(NUM_TYPES, 1.0, 3.0),
    )
    .validate(20, |alpha, beta, edge| {
        expect_small(alpha.pressure - beta.pressure, 0.1);

        // By U = 1.5 PV, since pressure and volume are similar, internal energy is also similar
        // 500 is a small number compared to the initial 20000
        expect_between(alpha.heat.0 - beta.heat.0, 1.0, 500.0);

        // Temperature has equalized a bit due to advection,
        // but still quite different and require convection later.
        assert!(alpha.temperature - beta.temperature > 100.0);

        // Despite temperature difference, pressure gradient still pushes heat to beta
        assert!(edge.last_heat.0 > 0.0, "alpha advecting hot fluid to beta");
        assert!(
            edge.last_typed_transfer[0].atob_transfer.0 > 0.0,
            "alpha advecting hot fluid to beta"
        );
    })
    .validate(100, |alpha, beta, edge| {
        expect_small(alpha.pressure - beta.pressure, 0.01);

        assert!(
            edge.last_heat.0 < 0.0,
            "beta warming up, gaining temperature but losing heat energy"
        );
        assert!(
            edge.last_typed_transfer[0].atob_transfer.0 < -0.0,
            "advecting back from beta to alpha"
        );
    });
}

struct Validate {
    app:   App,
    alpha: Entity,
    beta:  Entity,
    edge:  Entity,
}

impl Validate {
    fn validate(
        mut self,
        steps: u32,
        check: impl FnOnce(fluid::Storage, fluid::Storage, fluid::Edge),
    ) -> Self {
        for step in 0..steps {
            self.app.update();

            if option_env!("FLUID_TEST_DEBUG_PRINT") == Some("progress") {
                println!("Step {step}:");
                debug_print(&self.app, self.alpha, self.beta, self.edge);
            }
        }

        if option_env!("FLUID_TEST_DEBUG_PRINT").is_some() {
            println!("=== FINAL ===");
            debug_print(&self.app, self.alpha, self.beta, self.edge);
        }

        check(
            self.app.world().get::<fluid::Storage>(self.alpha).unwrap().clone(),
            self.app.world().get::<fluid::Storage>(self.beta).unwrap().clone(),
            self.app.world().get::<fluid::Edge>(self.edge).unwrap().clone(),
        );
        self
    }
}

fn do_test(
    set_types: impl FnOnce(&mut [fluid::TypeDef]),
    alpha_storage: fluid::Storage,
    beta_storage: fluid::Storage,
    edge: fluid::Edge,
) -> Validate {
    let mut app = App::new();
    app.add_plugins((time::TimePlugin, view::Plug, fluid::Plug));
    app.insert_resource(time::TimeUpdateStrategy::FixedTimesteps(1));
    app.insert_resource(fluid::Conf { transfer_timestep: 1 });
    {
        let mut types = default_types();
        set_types(&mut types);
        app.insert_resource(fluid::Types { types });
    }

    let alpha = app.world_mut().spawn(alpha_storage).id();
    let beta = app.world_mut().spawn(beta_storage).id();
    let edge = app.world_mut().spawn((edge, fluid::EdgeAlpha(alpha), fluid::EdgeBeta(beta))).id();

    Validate { app, alpha, beta, edge }
}

fn debug_print(app: &App, alpha: Entity, beta: Entity, edge: Entity) {
    debug_print_storage("Alpha", app.world().get(alpha).unwrap());
    debug_print_storage("Beta", app.world().get(beta).unwrap());
    debug_print_edge(app.world().get(edge).unwrap());
    println!();
}

fn debug_print_storage(
    title: &str,
    &fluid::Storage { pressure, mass, heat, temperature, ref types, .. }: &fluid::Storage,
) {
    println!("{title}: pressure({pressure}) mass({mass}) {heat:?} temperature({temperature})");

    for (ty, &fluid::TypedStorage { moles, molar_conc, proportion }) in types.iter().enumerate() {
        println!("  Type {ty:?}: {moles:?}, molar_conc({molar_conc}) proportion({proportion})");
    }
}

fn debug_print_edge(edge: &fluid::Edge) {
    println!("Heat flow: {}", print_flow(edge.last_heat.0));

    for (ty, typed) in edge.last_typed_transfer.iter().enumerate() {
        println!("  Type {ty:?}: {}", print_flow(typed.atob_transfer.0));
    }
}

fn print_flow(v: f32) -> impl fmt::Display {
    match v.partial_cmp(&0.0) {
        Some(Ordering::Less) => format!("b->a: {v:?}"),
        Some(Ordering::Greater) => format!("a->b: {v:?}"),
        Some(Ordering::Equal) => format!("none: {v:?}"),
        None => panic!("found {v:?} value"),
    }
}

#[track_caller]
fn expect_float(actual: f32, expect: f32) {
    if expect.is_nan() {
        assert!(actual.is_nan(), "expect {actual:?} to be nan");
    } else if expect.is_infinite() {
        assert!(
            actual.is_infinite() && actual.signum() == expect.signum(),
            "expect {actual:?} to be {expect:?}"
        );
    } else if expect == 0.0 {
        assert!(actual.abs() < 1e-4, "expect {actual:?} to be zero");
    } else {
        assert!(
            (actual - expect).abs() <= (expect * 1e-4).abs(),
            "got {actual:?}, expected {expect:?}",
        );
    }
}

#[track_caller]
fn expect_float_near(actual: f32, expect: f32, threshold: f32) {
    assert!(expect.is_finite());
    if (actual - expect).abs() > threshold {
        panic!("got {actual:?}, expected {expect:?} within {threshold}");
    }
}

#[track_caller]
fn expect_small(actual: f32, max_abs: f32) {
    assert!(actual.abs() < max_abs, "got abs({actual:?}), should be smaller than {max_abs}");
}

#[track_caller]
fn expect_between(actual: f32, min: f32, max: f32) {
    assert!(min < actual && actual < max, "got {actual:?}, should be between {min:?} and {max:?}");
}
