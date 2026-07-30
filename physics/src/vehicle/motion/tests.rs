use std::any::type_name;
use std::marker::PhantomData;

use bevy::app::{self, App};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{EntityCommand, ResMut, Single};
use bevy::math::{Vec2, Vec3};
use bevy::time;
use traffloat_proto::proto::AlphaOrBeta;

use crate::graph::{building, conduit, corridor, edge};
use crate::util::testing::{configure_logging, expect_float, expect_vec3};
use crate::util::{self, Alpha, AlphaBeta, Beta, Which};
use crate::{cleanup, fluid, persist, vehicle, view};

fn new_test(setup: TestSetup) -> Test {
    let mut app = App::new();
    if option_env!("RUST_LOG").is_some() {
        configure_logging(&mut app);
    }
    app.insert_resource(time::TimeUpdateStrategy::FixedTimesteps(1));
    app.init_resource::<vehicle::Conf>();
    app.init_resource::<vehicle::Types>();
    app.add_plugins((
        time::TimePlugin,
        cleanup::Plug,
        view::Plug,
        persist::Plug,
        fluid::Plug,
        vehicle::motion::Plug,
    ));

    util::configure_enum_system_set::<vehicle::SystemSets>(&mut app, app::FixedUpdate);

    app.update();

    let corridor = {
        let mut corridor = app.world_mut().spawn_empty();
        corridor.reborrow_scope(|e| {
            corridor::SpawnCommand {
                name:               Some(String::new()),
                endpoint_positions: AlphaBeta { alpha: Vec2::ZERO, beta: Vec2::new(1000.0, 0.0) },
                radius:             5.0,
                wall_thickness:     1.0,
            }
            .apply(e);
        });
        corridor.id()
    };

    let alpha_building = setup
        .has_alpha_building
        .then(|| make_building(&mut app, Vec2::new(-10.0, 0.0), corridor, Alpha));
    let beta_building = setup
        .has_beta_building
        .then(|| make_building(&mut app, Vec2::new(1010.0, 0.0), corridor, Beta));

    let rail = {
        let mut rail = app.world_mut().spawn_empty();
        rail.reborrow_scope(|e| make_rail(corridor).apply(e));
        rail.id()
    };

    make_vehicle_type().run(app.world_mut());

    let mut test = Test { app, corridor, alpha_building, beta_building, rail };

    test.app.update();

    test
}

fn make_building(app: &mut App, position: Vec2, corridor: Entity, which: impl Which) -> Entity {
    let mut building = app.world_mut().spawn_empty();
    building.reborrow_scope(|e| {
        building::SpawnCommand { name: String::new(), position, radius: 10.0, wall_thickness: 1.0 }
            .apply(e);
    });
    let building = building.id();

    let edge = app.world_mut().spawn_empty();
    edge::SpawnCommand { building, corridor, open: true, which }.apply(edge);

    building
}

fn make_rail(corridor: Entity) -> conduit::SpawnCommand {
    conduit::SpawnCommand {
        corridor,
        name: String::new(),
        radius: 2.0,
        typed: conduit::TypedSpawn::VehicleRail {
            rail:         vehicle::Rail {
                gauge_size:  vehicle::def::GaugeSize(1, 1),
                electrified: false,
                max_speed:   10.0,
            },
            reserved_dir: None,
        },
    }
}

fn make_vehicle_type() -> vehicle::AddTypeCommand {
    vehicle::AddTypeCommand {
        def: vehicle::TypeDef {
            name:           "test".into(),
            physical:       vehicle::def::Physical {
                mass:   200.0,
                gauge:  vehicle::def::GaugeSize(1, 1),
                volume: 10.0,
                length: 2.0,
            },
            motion:         vehicle::def::Motion {
                propulsion:       vehicle::Propulsion {
                    inputs:    Vec::new(),
                    outputs:   vec![vehicle::propulsion::ForceOutput { max_force: 1000.0 }.into()],
                    catalysts: Vec::new(),
                },
                max_speed:        20.0, // overridden by rail max speed
                max_braking:      2000.0,
                drag_coefficient: 0.3,
            },
            compartments:   [vehicle::def::Compartment {
                name:                  String::new(),
                volume:                100.0,
                passenger_slots:       1,
                vent_area:             0.0,
                vent_resistance_recip: 1.0,
            }]
            .into(),
            operator_slots: [].into(),
        },
    }
}

fn spawn_vehicle<WhichVehicle: Component + Default>(
    location: vehicle::Location,
    app: &mut App,
) -> Entity {
    app.insert_resource(DummyPathfinderOnce::<WhichVehicle>(None, PhantomData));
    app.add_systems(
        app::FixedUpdate,
        dummy_pathfinder_system::<WhichVehicle>.in_set(vehicle::SystemSets::Pathfinding),
    );

    let mut vehicle = app.world_mut().spawn(WhichVehicle::default());
    vehicle.reborrow_scope(|e| {
        vehicle::SpawnCommand { name: Some(String::new()), ty: vehicle::TypeId(0), location }
            .apply(e);
    });
    vehicle.id()
}

#[derive(Resource)]
struct DummyPathfinderOnce<WhichVehicle: Component>(
    Option<vehicle::motion::Intent>,
    PhantomData<WhichVehicle>,
);

fn dummy_pathfinder_system<WhichVehicle: Component>(
    mut once: ResMut<DummyPathfinderOnce<WhichVehicle>>,
    vehicle_query: Option<Single<&mut vehicle::motion::Intent, With<WhichVehicle>>>,
) {
    if let Some(mut vehicle_query) = vehicle_query {
        let new_intent = once
            .0
            .take()
            .expect("DummyPathfinderOnce must be set exactly once before each app.update()");
        **vehicle_query = new_intent;
    } else {
        assert!(once.0.is_none(), "DummyPathfinderOnce was set but vehicle does not exist");
    }
}

struct TestSetup {
    has_alpha_building: bool,
    has_beta_building:  bool,
}

struct Test {
    app:            App,
    corridor:       Entity,
    alpha_building: Option<Entity>,
    beta_building:  Option<Entity>,
    rail:           Entity,
}

impl Test {
    fn building<Ab: Which>(&self, which: Ab) -> Entity {
        match which.proto() {
            AlphaOrBeta::Alpha => self.alpha_building.unwrap(),
            AlphaOrBeta::Beta => self.beta_building.unwrap(),
        }
    }

    #[track_caller]
    fn assert_desired(&self, vehicle: Entity, expect: vehicle::propulsion::Desired) {
        let actual = self.app.world().get::<vehicle::propulsion::Desired>(vehicle).unwrap();
        match (&expect, actual) {
            (
                vehicle::propulsion::Desired::Stationary,
                vehicle::propulsion::Desired::Stationary,
            ) => {}
            (
                &vehicle::propulsion::Desired::Building { interior_pos: expect_pos },
                &vehicle::propulsion::Desired::Building { interior_pos: actual_pos },
            ) => {
                expect_vec3(actual_pos, expect_pos);
            }
            (
                &vehicle::propulsion::Desired::Rail { speed_from_alpha: expect_displace },
                &vehicle::propulsion::Desired::Rail { speed_from_alpha: actual_displace },
            ) => {
                expect_float(actual_displace, expect_displace);
            }
            _ => panic!("Expected desired propulsion {expect:?}, got {actual:?}"),
        }
    }

    #[track_caller]
    fn assert_location(&self, vehicle: Entity, expect: vehicle::Location) {
        let actual = self.app.world().get::<vehicle::Location>(vehicle).unwrap();
        match (expect, *actual) {
            (
                vehicle::Location::Building {
                    building: expect_building,
                    interior_pos: expect_pos,
                    speed: expect_speed,
                },
                vehicle::Location::Building {
                    building: actual_building,
                    interior_pos: actual_pos,
                    speed: actual_speed,
                },
            ) => {
                assert_eq!(actual_building, expect_building);
                expect_vec3(actual_pos, expect_pos);
                expect_vec3(actual_speed, expect_speed);
            }
            (
                vehicle::Location::Rail {
                    conduit: expect_rail,
                    distance_from_alpha: expect_pos,
                    speed_from_alpha: expect_speed,
                },
                vehicle::Location::Rail {
                    conduit: actual_rail,
                    distance_from_alpha: actual_pos,
                    speed_from_alpha: actual_speed,
                },
            ) => {
                assert_eq!(actual_rail, expect_rail);
                expect_float(actual_pos, expect_pos);
                expect_float(actual_speed, expect_speed);
            }
            _ => panic!("Expected location {expect:?}, got {actual:?}"),
        }
    }

    #[track_caller]
    fn assert_reserved_direction(&self, expect: Option<vehicle::rail::ReservedDirection>) {
        let actual = self.app.world().get::<vehicle::rail::Reservation>(self.rail).unwrap();
        match (expect, &actual.inner) {
            (None, None) => {}
            (Some(expect_dir), Some(actual_inner)) => {
                assert_eq!(actual_inner.direction, expect_dir);
            }
            _ => panic!("Expected reserved direction {expect:?}, got {actual:?}"),
        }
    }

    fn expect_pathfind_once<WhichVehicle: Component>(
        &mut self,
        new_intent: vehicle::motion::Intent,
    ) {
        let mut once = self.app.world_mut().resource_mut::<DummyPathfinderOnce<WhichVehicle>>();
        assert!(
            once.0.is_none(),
            "DummyPathfinderOnce<{}> was already set and not consumed",
            type_name::<WhichVehicle>()
        );
        once.0 = Some(new_intent);
    }
}

#[test]
fn rule_a_building_local() {
    #[derive(Component, Default)]
    struct MainVehicle;

    let mut test = new_test(TestSetup { has_alpha_building: true, has_beta_building: false });

    let vehicle = spawn_vehicle::<MainVehicle>(
        vehicle::Location::Building {
            building:     test.alpha_building.unwrap(),
            interior_pos: Vec3::ZERO,
            speed:        Vec3::ZERO,
        },
        &mut test.app,
    );
    test.expect_pathfind_once::<MainVehicle>(vehicle::motion::Intent::Building {
        target:               test.alpha_building.unwrap(),
        stop_at_interior_pos: Vec3::new(5.0, 0.0, 0.0),
    });
    test.app.update();

    test.assert_desired(
        vehicle,
        vehicle::propulsion::Desired::Building { interior_pos: Vec3::new(5.0, 0.0, 0.0) },
    );
    test.assert_reserved_direction(None);
}

#[test]
fn rule_b_building_to_rail_alpha_pursuit() {
    rule_b_building_to_rail_pursuit(Alpha, Vec3::new(10.0, 0.0, 0.0));
}

#[test]
fn rule_b_building_to_rail_beta_pursuit() {
    rule_b_building_to_rail_pursuit(Beta, Vec3::new(-10.0, 0.0, 0.0));
}

fn rule_b_building_to_rail_pursuit(which: impl Which, expect_interior_pos: Vec3) {
    #[derive(Component, Default)]
    struct MainVehicle;

    let mut test = new_test(TestSetup { has_alpha_building: true, has_beta_building: true });

    let vehicle = spawn_vehicle::<MainVehicle>(
        vehicle::Location::Building {
            building:     test.building(which),
            interior_pos: Vec3::ZERO,
            speed:        Vec3::ZERO,
        },
        &mut test.app,
    );
    test.expect_pathfind_once::<MainVehicle>(vehicle::motion::Intent::Rail {
        target_rail:      test.rail,
        through_building: test.building(which),
    });
    test.app.update();

    test.assert_desired(
        vehicle,
        vehicle::propulsion::Desired::Building { interior_pos: expect_interior_pos },
    );
    test.assert_reserved_direction(None);
}

#[test]
fn rule_b_building_to_rail_alpha_clear_and_block() {
    rule_b_building_to_rail_clear_and_block(Alpha);
}

#[test]
fn rule_b_building_to_rail_beta_clear_and_block() { rule_b_building_to_rail_clear_and_block(Beta); }

fn rule_b_building_to_rail_clear_and_block(which: impl Which) {
    #[derive(Component, Default)]
    struct VehicleAhead;

    #[derive(Component, Default)]
    struct VehicleBehind;

    let mut test = new_test(TestSetup { has_alpha_building: true, has_beta_building: true });

    rule_b_building_to_rail_clear_and_block_round_one::<VehicleAhead>(which, &mut test);
    rule_b_building_to_rail_clear_and_block_round_two::<VehicleAhead, VehicleBehind>(
        which, &mut test,
    );
}

// Round 1: rail clear, enter directly unobstructed.
fn rule_b_building_to_rail_clear_and_block_round_one<VehicleAhead: Component + Default>(
    which: impl Which,
    test: &mut Test,
) {
    let vehicle = spawn_vehicle::<VehicleAhead>(
        vehicle::Location::Building {
            building:     test.building(which),
            interior_pos: which.select_with(Vec3::new(9.1, 0.0, 0.0), Vec3::new(-9.1, 0.0, 0.0)),
            speed:        Vec3::ZERO,
        },
        &mut test.app,
    );
    test.expect_pathfind_once::<VehicleAhead>(vehicle::motion::Intent::Rail {
        target_rail:      test.rail,
        through_building: test.building(which),
    });
    test.app.update();

    test.assert_location(
        vehicle,
        vehicle::Location::Rail {
            conduit:             test.rail,
            distance_from_alpha: which.select_with(-1.0, 1001.0),
            speed_from_alpha:    which
                .negate_if_beta(vehicle::Conf::default().standard_drifting_speed),
        },
    );
    let dir = vehicle::rail::ReservedDirection::from_entry(which.proto());
    test.assert_reserved_direction(Some(dir));

    test.expect_pathfind_once::<VehicleAhead>(vehicle::motion::Intent::Building {
        target:               test.building(which.other()),
        stop_at_interior_pos: Vec3::ZERO,
    });
    test.app.update();

    test.assert_desired(
        vehicle,
        vehicle::propulsion::Desired::Rail { speed_from_alpha: which.negate_if_beta(10.0) },
    );
}

// Round 2: rail blocked by entering vehicle, unable to enter.
fn rule_b_building_to_rail_clear_and_block_round_two<
    VehicleAhead: Component + Default,
    VehicleBehind: Component + Default,
>(
    entry: impl Which,
    test: &mut Test,
) {
    let initial_location = vehicle::Location::Building {
        building:     test.building(entry),
        interior_pos: entry.select_with(Vec3::new(9.1, 0.0, 0.0), Vec3::new(-9.1, 0.0, 0.0)),
        speed:        Vec3::ZERO,
    };
    let second_vehicle = spawn_vehicle::<VehicleBehind>(initial_location, &mut test.app);

    test.expect_pathfind_once::<VehicleAhead>(vehicle::motion::Intent::Building {
        target:               test.building(entry.other()),
        stop_at_interior_pos: Vec3::ZERO,
    });
    test.expect_pathfind_once::<VehicleBehind>(vehicle::motion::Intent::Rail {
        target_rail:      test.rail,
        through_building: test.building(entry),
    });
    test.app.update();

    test.assert_location(second_vehicle, initial_location);
}

#[test]
fn rule_c_rail_local_atob_to_endpoint_dead_end() { rule_c_rail_local_to_endpoint_dead_end(Alpha); }

#[test]
fn rule_c_rail_local_btoa_to_endpoint_dead_end() { rule_c_rail_local_to_endpoint_dead_end(Beta); }

fn rule_c_rail_local_to_endpoint_dead_end<Ab: Which>(which: Ab) {
    #[derive(Component, Default)]
    struct MainVehicle;

    let mut test = new_test(TestSetup { has_alpha_building: true, has_beta_building: true });

    spawn_vehicle::<MainVehicle>(
        vehicle::Location::Rail {
            conduit:             test.rail,
            distance_from_alpha: 500.0,
            speed_from_alpha:    0.0,
        },
        &mut test.app,
    );
}

#[test]
fn rule_c_rail_local_atob_to_endpoint_building() {}

#[test]
fn rule_c_rail_local_btoa_to_endpoint_building() {}

#[test]
fn rule_c_rail_local_atob_blocked_by_vehicle() {}

#[test]
fn rule_c_rail_local_btoa_blocked_by_vehicle() {}

#[test]
fn rule_d_rail_to_building_alpha() {}

#[test]
fn rule_d_rail_to_building_beta() {}
