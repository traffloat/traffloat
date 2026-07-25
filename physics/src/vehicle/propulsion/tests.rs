use std::time::Duration;

use bevy::app::App;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Command, EntityCommand};
use bevy::math::Vec2;
use bevy::time;

use crate::graph::{conduit, corridor};
use crate::util::testing::{configure_logging, expect_float, expect_float_near};
use crate::util::{AlphaBeta, duration_to_timesteps};
use crate::vehicle::Propulsion;
use crate::{cleanup, fluid, persist, vehicle, view};

fn new_test() -> Test {
    let mut app = App::new();
    configure_logging(&mut app);
    app.insert_resource(time::TimeUpdateStrategy::FixedTimesteps(1));
    app.init_resource::<vehicle::Conf>();
    app.init_resource::<vehicle::Types>();
    app.add_plugins((
        time::TimePlugin,
        cleanup::Plug,
        view::Plug,
        persist::Plug,
        fluid::Plug,
        vehicle::propulsion::Plug,
    ));
    app.update();

    let air = fluid::AddTypeCommand {
        def: fluid::TypeDef {
            name:                 "air".into(),
            molar_heat_capacity:  1.0,
            advective_fluidity:   1.0,
            diffusive_fluidity:   1.0,
            molar_density:        1.0,
            thermal_conductivity: 1e-4,
            optical_extinction:   [0.0; 3],
        },
    }
    .run(app.world_mut());

    let mut corridor = app.world_mut().spawn_empty();
    corridor.reborrow_scope(|e| {
        corridor::SpawnCommand {
            name:               Some("".into()),
            endpoint_positions: AlphaBeta { alpha: Vec2::ZERO, beta: Vec2::new(1000.0, 0.0) },
            radius:             5.0,
            wall_thickness:     1.0,
        }
        .apply(e)
    });
    let mut fluid = corridor.get_mut::<fluid::Storage>().unwrap();
    fluid.set_heat(fluid::Energy(3e7));
    fluid.set_fluid(air, fluid::Moles(1e5));
    let corridor = corridor.id();

    let mut rail = app.world_mut().spawn_empty();
    rail.reborrow_scope(|e| {
        conduit::SpawnCommand {
            corridor,
            name: "".into(),
            radius: 2.0,
            typed: conduit::TypedSpawn::VehicleRail {
                rail:         vehicle::Rail {
                    gauge_size:  vehicle::def::GaugeSize(1, 1),
                    electrified: false,
                    max_speed:   10.0,
                },
                reserved_dir: Some(vehicle::rail::ReservedDirection::AlphaToBeta),
            },
        }
        .apply(e)
    });
    let rail = rail.id();

    let vehicle_ty = vehicle::AddTypeCommand {
        def: vehicle::TypeDef {
            name:           "test".into(),
            physical:       vehicle::def::Physical {
                mass:   200.0,
                gauge:  vehicle::def::GaugeSize(1, 1),
                volume: 10.0,
                length: 2.0,
            },
            motion:         vehicle::def::Motion {
                propulsion:       Propulsion {
                    inputs:    Vec::new(),
                    outputs:   vec![vehicle::propulsion::ForceOutput { max_force: 1000.0 }.into()],
                    catalysts: Vec::new(),
                },
                max_speed:        20.0, // overridden by rail max speed
                max_braking:      2000.0,
                drag_coefficient: 0.3,
            },
            compartments:   [vehicle::def::Compartment {
                name:                  "".into(),
                volume:                100.0,
                passenger_slots:       1,
                vent_area:             0.0,
                vent_resistance_recip: 1.0,
            }]
            .into(),
            operator_slots: [].into(),
        },
    }
    .run(app.world_mut());

    let mut vehicle = app.world_mut().spawn_empty();
    vehicle.reborrow_scope(|e| {
        vehicle::SpawnCommand {
            name:     Some("".into()),
            ty:       vehicle_ty,
            location: vehicle::Location::Rail {
                conduit:             rail,
                distance_from_alpha: 500.0,
                speed_from_alpha:    0.0,
            },
        }
        .apply(e);
    });
    let vehicle = vehicle.id();

    app.update();

    Test { app, corridor, rail, vehicle }
}

struct Test {
    app:      App,
    corridor: Entity,
    rail:     Entity,
    vehicle:  Entity,
}

impl Test {
    fn assert_displace_speed(&self, expected_displace: f32, expected_speed: f32) {
        let location = self.app.world().get::<vehicle::Location>(self.vehicle).unwrap();
        let &vehicle::Location::Rail { conduit, distance_from_alpha, speed_from_alpha } = location
        else {
            panic!("Vehicle must be on a rail");
        };
        assert_eq!(conduit, self.rail);

        expect_float(distance_from_alpha, expected_displace);
        expect_float(speed_from_alpha, expected_speed);
    }

    fn assert_efficiency(&self, expected_efficiency: f32) {
        let status = self.app.world().get::<vehicle::propulsion::Status>(self.vehicle).unwrap();
        expect_float_near(status.propulsion_efficiency, expected_efficiency, 1e-4);
    }

    fn progress(&mut self, steps: u32) {
        for _ in 0..steps {
            self.app.update();
        }
    }

    fn set_desired(&mut self, desired_speed: f32) {
        let mut propulsion =
            self.app.world_mut().get_mut::<vehicle::propulsion::Desired>(self.vehicle).unwrap();
        *propulsion = vehicle::propulsion::Desired::Rail { speed_from_alpha: desired_speed };
    }
}

#[test]
fn test_baseline() {
    let mut test = new_test();
    test.assert_displace_speed(500.0, 0.0);
    test.assert_efficiency(0.0);

    test.set_desired(10.0);
    test.progress(duration_to_timesteps(Duration::from_secs(10)));
    test.assert_displace_speed(589.8961, 10.0);

    test.set_desired(-10.0);
    test.progress(duration_to_timesteps(Duration::from_secs(10)));
    test.assert_displace_speed(509.7590, -10.0);

    test.set_desired(0.0);
    test.progress(duration_to_timesteps(Duration::from_secs(10)));
    test.assert_displace_speed(506.5631, 0.0);
}
