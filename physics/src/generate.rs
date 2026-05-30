use std::f32::consts::PI;

use bevy::ecs::entity::Entity;
use bevy::ecs::system::EntityCommand;
use bevy::ecs::world::World;

use crate::graph::facility::{self, Blueprint, blueprint};
use crate::graph::{self, building, corridor, edge};
use crate::util::{Alpha, AlphaBeta, Beta, Which};
use crate::{WorldObject, fluid, reactor, view};

const STANDARD_WALL_THICKNESS: f32 = 0.5;

pub struct Config {}

/// Generate a basic physics world.
pub fn generate(world: &mut World, _: Config) {
    let fluids = gen_fluid_types(world);
    let reactors = gen_reactor_types(world, &fluids);
    let facilities = gen_facility_types(world, &reactors);
    let std = StandardTypes { fluids, reactors, facilities };

    let core = gen_core(world, &std);
    let garden = gen_garden(world, &std);
    gen_corridor(world, &std, AlphaBeta { alpha: core, beta: garden }, 1.1);
}

struct StandardTypes {
    fluids:     StandardFluidTypes,
    reactors:   StandardReactorTypes,
    facilities: StandardFacilityTypes,
}

struct StandardFluidTypes {
    nitrogen:       fluid::TypeId,
    oxygen:         fluid::TypeId,
    carbon_dioxide: fluid::TypeId,
    water:          fluid::TypeId,

    atmosphere:  Vec<(fluid::TypeId, f32)>,
    temperature: f32,
}

fn gen_fluid_types(world: &mut World) -> StandardFluidTypes {
    let mut types = world.resource_mut::<fluid::Types>();
    let nitrogen = types.push(fluid::TypeDef {
        name:                 "Nitrogen".into(),
        molar_heat_capacity:  20.800,
        molar_density:        28.014,
        advective_fluidity:   0.4,
        diffusive_fluidity:   0.08,
        thermal_conductivity: 0.026,
        optical_extinction:   [1e-3, 1e-3, 1e-3],
    });
    let oxygen = types.push(fluid::TypeDef {
        name:                 "Oxygen".into(),
        molar_heat_capacity:  21.000,
        molar_density:        31.998,
        advective_fluidity:   0.344,
        diffusive_fluidity:   0.0748,
        thermal_conductivity: 0.027,
        optical_extinction:   [1e-3, 1e-3, 1e-3],
    });
    let carbon_dioxide = types.push(fluid::TypeDef {
        name:                 "Carbon dioxide".into(),
        molar_heat_capacity:  28.460,
        molar_density:        44.009,
        advective_fluidity:   0.475,
        diffusive_fluidity:   0.0635,
        thermal_conductivity: 0.017,
        optical_extinction:   [1e-3, 1e-3, 1e-3],
    });
    let water = types.push(fluid::TypeDef {
        name:                 "Water".into(),
        molar_heat_capacity:  75.327,
        molar_density:        18.015,
        advective_fluidity:   0.1,
        diffusive_fluidity:   0.01,
        thermal_conductivity: 0.6,
        optical_extinction:   [0.055, 0.53, 0.8],
    });
    StandardFluidTypes {
        nitrogen,
        oxygen,
        carbon_dioxide,
        water,
        atmosphere: [(nitrogen, 0.78), (oxygen, 0.21), (carbon_dioxide, 0.01)].into(),
        temperature: 293.15,
    }
}

struct StandardReactorTypes {
    /// Ports:
    /// 0. Ambient fluid
    /// 1. Water input
    garden: reactor::TypeId,
}

fn gen_reactor_types(world: &mut World, std_fluids: &StandardFluidTypes) -> StandardReactorTypes {
    let mut types = world.resource_mut::<reactor::Types>();
    let garden = types.push(reactor::TypeDef {
        inputs:    [reactor::Input::Fluid(reactor::FluidInput {
            storage:        reactor::FluidStorageRef(0),
            ty:             std_fluids.carbon_dioxide,
            max_rate:       fluid::Moles(0.1),
            conc_threshold: reactor::Threshold {
                curve:         reactor::Curve::Linear {
                    min_input:      0.0,
                    max_input:      0.1,
                    min_multiplier: 0.0,
                    max_multiplier: 1.0,
                },
                modifier_type: reactor::ThresholdModifierType::Maximum,
            },
        })]
        .into(),
        outputs:   [reactor::Output::Fluid(reactor::FluidOutput {
            storage:  reactor::FluidStorageRef(0),
            ty:       std_fluids.oxygen,
            max_rate: fluid::Moles(0.1),
        })]
        .into(),
        catalysts: [
            reactor::Catalyst::Fluid(reactor::FluidCatalyst {
                storage:        reactor::FluidStorageRef(1),
                ty:             std_fluids.water,
                conc_threshold: reactor::Threshold {
                    curve:         reactor::Curve::Linear {
                        min_input:      0.0,
                        max_input:      0.5,
                        min_multiplier: 0.0,
                        max_multiplier: 1.0,
                    },
                    modifier_type: reactor::ThresholdModifierType::Maximum,
                },
            }),
            reactor::Catalyst::Temperature(reactor::TemperatureCatalyst {
                storage:        reactor::FluidStorageRef(0),
                temp_threshold: reactor::Threshold {
                    curve:         reactor::Curve::Gaussian {
                        optimal_input:      303.0,
                        input_scale:        15.0,
                        optimal_multiplier: 1.0,
                        minimal_multiplier: 0.0,
                    },
                    modifier_type: reactor::ThresholdModifierType::Multiplier,
                },
            }),
        ]
        .into(),
    });
    StandardReactorTypes { garden }
}

struct StandardFacilityTypes {
    garden: Entity,
}

fn gen_facility_types(
    world: &mut World,
    std_reactor: &StandardReactorTypes,
) -> StandardFacilityTypes {
    let garden = world
        .spawn((
            WorldObject,
            graph::FacilityTypeDef {
                display:   "Garden".into(),
                volume:    300.0,
                blueprint: Blueprint {
                    reactor: Some(blueprint::Reactor {
                        ty:    std_reactor.garden,
                        ports: blueprint::Ports {
                            fluid_storages: [blueprint::FluidStoragePort::default()
                                .with(blueprint::FluidStoragePortType::Ambient)]
                            .into(),
                        },
                    }),
                    ..Default::default()
                },
            },
        ))
        .id();
    StandardFacilityTypes { garden }
}

fn gen_core(world: &mut World, std: &StandardTypes) -> Entity {
    let mut building = world.spawn((WorldObject,));
    building.reborrow_scope(|building| {
        building::SpawnCommand {
            building: graph::Building {
                name:           "Core".into(),
                position:       (0.0, 0.0).into(),
                radius:         15.0,
                wall_thickness: 0.8,
                ambient_volume: const { sphere_volume(15.0) },
            },
        }
        .apply(building);
    });

    let building_id = building.id();
    std.fluids.fill_atmosphere(world, building_id);
    building_id
}

fn gen_garden(world: &mut World, std: &StandardTypes) -> Entity {
    let mut building = world.spawn((WorldObject,));
    building.reborrow_scope(|building| {
        building::SpawnCommand {
            building: graph::Building {
                name:           "Garden".into(),
                position:       (40.0, 0.0).into(),
                radius:         6.0,
                wall_thickness: STANDARD_WALL_THICKNESS,
                ambient_volume: const { sphere_volume(6.0) },
            },
        }
        .apply(building);
    });

    let building_id = building.id();
    std.fluids.fill_atmosphere(world, building_id);

    let facility = world.spawn((
        WorldObject,
        reactor::Facility {
            id:             std.reactors.garden,
            efficiency_cap: 1.0,
            ports:          reactor::Ports { fluid_storages: [Some(building_id), None].into() },
        },
    ));
    facility::SpawnCommand { building: building_id, ty: std.facilities.garden }.apply(facility);

    building_id
}

fn gen_corridor(world: &mut World, std: &StandardTypes, endpoints: AlphaBeta<Entity>, radius: f32) {
    let (building_centers, building_radii) = endpoints
        .map(|building| {
            let building =
                world.get::<graph::Building>(building).expect("endpoints must be buildings");
            (building.position, building.radius)
        })
        .unzip();
    let dir = building_centers.atob().normalize_or_zero();
    let endpoint_positions = AlphaBeta {
        alpha: building_centers.alpha + dir * building_radii.alpha,
        beta:  building_centers.beta - dir * building_radii.beta,
    };

    let mut corridor = world.spawn((WorldObject,));
    corridor.reborrow_scope(|corridor| {
        corridor::SpawnCommand {
            name: None,
            endpoint_positions,
            radius,
            length: endpoint_positions.net_diff().length(),
            wall_thickness: STANDARD_WALL_THICKNESS,
            ambient_area: circle_area(radius),
        }
        .apply(corridor);
    });

    // TODO spawn pipes

    let corridor_id = corridor.id();
    std.fluids.fill_atmosphere(world, corridor_id);

    spawn_edge(Alpha, world, endpoints, corridor_id);
    spawn_edge(Beta, world, endpoints, corridor_id);
}

fn spawn_edge<Ab: Which>(
    which: Ab,
    world: &mut World,
    endpoints: AlphaBeta<Entity>,
    corridor: Entity,
) {
    let mut edge = world.spawn_empty();
    edge.reborrow_scope(|edge| {
        edge::SpawnCommand { building: which.select(endpoints), corridor, which, open: true }
            .apply(edge);
    });
}

impl StandardFluidTypes {
    fn fill_atmosphere(&self, world: &mut World, building: Entity) {
        let mut building = world.entity_mut(building);
        let mut storage =
            building.get_mut::<fluid::Storage>().expect("building must have fluid storage");
        for &(ty, fraction) in &self.atmosphere {
            let moles = fluid::Moles(fraction * storage.volume);
            storage.set_fluid(ty, moles);
        }

        fluid::SetTemperatureCommand { temperature: self.temperature }.apply(building);
    }
}

const fn sphere_volume(radius: f32) -> f32 { 4.0 / 3.0 * PI * radius * radius * radius }

const fn circle_area(radius: f32) -> f32 { 0.5 * PI * radius * radius }
