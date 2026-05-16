use std::cmp::Ordering;
use std::time::Duration;
use std::{fmt, iter};

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::time;

use crate::fluid;

fn default_types() -> Vec<fluid::TypeDef> {
    iter::repeat_with(|| fluid::TypeDef {
        molar_heat_capacity:  1.0,
        advective_fluidity:   1.0,
        diffusive_fluidity:   1.0,
        molar_density:        1.0,
        thermal_conductivity: 1e-4,
    })
    .take(16)
    .collect()
}

#[test]
fn test_empty() {
    let (alpha, beta, edge) = do_test(
        |_| {},
        fluid::Storage::vacuum(100.0),
        fluid::Storage::vacuum(100.0),
        fluid::Edge::new(1.0, 10.0),
        20,
    );

    expect_float(alpha.pressure, 0.0);
    expect_float(beta.pressure, 0.0);

    expect_float(alpha.mass, 0.0);
    expect_float(beta.mass, 0.0);

    expect_float(edge.last_heat.0, 0.0);
}

#[test]
fn test_equilibrium_big_small() {
    let (alpha, beta, edge) = do_test(
        |_| {},
        fluid::Storage::vacuum(100.0)
            .with_heat(fluid::Energy(30000.0))
            .with_fluid(fluid::TypeId(0), 100.0),
        fluid::Storage::vacuum(10.0)
            .with_heat(fluid::Energy(3000.0))
            .with_fluid(fluid::TypeId(0), 10.0),
        fluid::Edge::new(1.0, 10.0),
        20,
    );

    expect_float(alpha.pressure, 2.34375);
    expect_float(beta.pressure, 2.34375);

    expect_float(alpha.mass, 100.0);
    expect_float(beta.mass, 10.0);

    expect_float(edge.last_heat.0, 0.0);
}

#[test]
fn test_diffusion_big_small() {
    let (alpha, beta, edge) = do_test(
        |_| {},
        fluid::Storage::vacuum(100.0)
            .with_heat(fluid::Energy(30000.0))
            .with_fluid(fluid::TypeId(0), 100.0),
        fluid::Storage::vacuum(10.0)
            .with_heat(fluid::Energy(3000.0))
            .with_fluid(fluid::TypeId(1), 10.0),
        fluid::Edge::new(1.0, 10.0),
        320,
    );

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
}

#[test]
fn test_convection_same_size() {
    let (alpha, beta, edge) = do_test(
        |_| {},
        fluid::Storage::vacuum(100.0)
            .with_heat(fluid::Energy(40000.0))
            .with_fluid(fluid::TypeId(0), 100.0),
        fluid::Storage::vacuum(100.0)
            .with_heat(fluid::Energy(20000.0))
            .with_fluid(fluid::TypeId(0), 100.0),
        fluid::Edge::new(1.0, 10.0),
        320,
    );
}

fn do_test(
    set_types: impl FnOnce(&mut [fluid::TypeDef]),
    alpha_storage: fluid::Storage,
    beta_storage: fluid::Storage,
    edge: fluid::Edge,
    steps: u32,
) -> (fluid::Storage, fluid::Storage, fluid::Edge) {
    let mut app = App::new();
    app.add_plugins((time::TimePlugin, fluid::Plug));
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

    for step in 0..steps {
        app.update();

        if option_env!("FLUID_TEST_DEBUG_PRINT") == Some("progress") {
            println!("Step {step}:");
            debug_print(&app, alpha, beta, edge);
        }
    }

    if option_env!("FLUID_TEST_DEBUG_PRINT").is_some() {
        println!("=== FINAL ===");
        debug_print(&app, alpha, beta, edge);
    }

    (
        app.world().get::<fluid::Storage>(alpha).unwrap().clone(),
        app.world().get::<fluid::Storage>(beta).unwrap().clone(),
        app.world().get::<fluid::Edge>(edge).unwrap().clone(),
    )
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

    for &fluid::TypedStorage { ty, moles, molar_conc, proportion } in types {
        println!("  Type {ty:?}: {moles:?}, molar_conc({molar_conc}) proportion({proportion})");
    }
}

fn debug_print_edge(edge: &fluid::Edge) {
    println!("Heat flow: {}", print_flow(edge.last_heat.0));

    for typed in &edge.last_typed_transfer {
        println!("  Type {:?}: {}", typed.ty, print_flow(typed.atob_transfer.0));
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
fn assert_small(actual: f32, max_abs: f32) {
    assert!(actual.abs() < max_abs, "expect abs({actual:?}) to be smaller than {max_abs}");
}
