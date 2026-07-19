#![allow(clippy::too_many_lines, reason = "this file contains nested hardcoded constants")]

use std::f32::consts::PI;
use std::time::Duration;

use bevy::ecs::entity::Entity;
use bevy::ecs::name::Name;
use bevy::ecs::system::EntityCommand;
use bevy::ecs::world::World;
use bevy::math::Vec3;
use enum_map::enum_map;

use crate::graph::facility::{self, Blueprint, blueprint};
use crate::graph::{self, building, conduit, connection, corridor, edge};
use crate::util::{Alpha, AlphaBeta, Beta, Which, duration_to_timesteps};
use crate::{WorldObject, fluid, reaction, reactor, resident, view};

const STANDARD_WALL_THICKNESS: f32 = 0.5;

pub struct Config {
    pub seed: u64,
}

/// Generate a basic physics world.
pub fn generate(world: &mut World, _: Config) {
    let fluids = gen_fluid_types(world);
    let attrs = gen_resident_attr_types(world);
    let reactors = gen_reactor_types(world, &fluids);
    let facilities = gen_facility_types(world, &reactors);
    let std = StandardTypes { fluids, attrs, reactors, facilities };
    gen_resident_ambient_interactions(world, &std);

    let core = gen_core(world, &std);
    let garden = gen_garden(world, &std);
    spawn_corridor(
        world,
        &std,
        AlphaBeta { alpha: core.building, beta: garden.building },
        1.1,
        |world, corridor| {
            let pipe =
                spawn_fluid_pipe(world, corridor, core.tank, garden.facility, "Water pipe", 0.1);
            connect_facility_pipe(world, core.tank, pipe);
        },
    );

    spawn_resident_in_building(world, core.building);
    spawn_resident_in_facility_slot(world, garden.facility, 0);
}

struct StandardTypes {
    fluids:     StandardFluidTypes,
    attrs:      StandardResidentAttrTypes,
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
        optical_extinction:   [0.8, 0.53, 0.055],
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
        inputs:    [reactor::Input::Fluid(reaction::input::Fluid {
            selector:       reactor::FluidPortSelector { port: 0 },
            ty:             std_fluids.carbon_dioxide,
            max_rate:       fluid::Moles(0.1),
            conc_threshold: reaction::Threshold {
                curve:         reaction::Curve::Linear {
                    min_input:      0.0,
                    max_input:      0.1,
                    min_multiplier: 0.0,
                    max_multiplier: 1.0,
                },
                modifier_type: reaction::ThresholdModifierType::Maximum,
            },
        })]
        .into(),
        outputs:   [reactor::Output::Fluid(reaction::output::Fluid {
            selector: reactor::FluidPortSelector { port: 0 },
            ty:       std_fluids.oxygen,
            max_rate: fluid::Moles(0.2),
        })]
        .into(),
        catalysts: [
            reactor::Catalyst::Fluid(reaction::catalyst::Fluid {
                selector:       reactor::FluidPortSelector { port: 1 },
                ty:             std_fluids.water,
                conc_threshold: reaction::Threshold {
                    curve:         reaction::Curve::Linear {
                        min_input:      0.0,
                        max_input:      0.5,
                        min_multiplier: 0.0,
                        max_multiplier: 1.0,
                    },
                    modifier_type: reaction::ThresholdModifierType::Maximum,
                },
            }),
            reactor::Catalyst::Temperature(reaction::catalyst::Temperature {
                selector:       reactor::FluidPortSelector { port: 0 },
                temp_threshold: reaction::Threshold {
                    curve:         reaction::Curve::Gaussian {
                        optimal_input:      303.0,
                        input_scale:        15.0,
                        optimal_multiplier: 1.0,
                        minimal_multiplier: 0.0,
                    },
                    modifier_type: reaction::ThresholdModifierType::Multiplier,
                },
            }),
        ]
        .into(),
    });
    StandardReactorTypes { garden }
}

struct StandardFacilityTypes {
    garden:     Entity,
    small_tank: Entity,
}

fn gen_facility_types(
    world: &mut World,
    std_reactor: &StandardReactorTypes,
) -> StandardFacilityTypes {
    let garden = world
        .spawn((
            WorldObject,
            Name::new("FacilityTypeDef Garden"),
            graph::FacilityTypeDef {
                display_name: "Garden".into(),
                volume:       300.0,
                sprite_id:    "facility/garden".into(),
                blueprint:    Blueprint {
                    reactor: Some(blueprint::Reactor {
                        ty:    std_reactor.garden,
                        ports: blueprint::Ports {
                            fluid_storages: [blueprint::FluidStoragePort::default()
                                .with(blueprint::FluidStoragePortType::Ambient)]
                            .into(),
                        },
                    }),
                    interaction_slots: vec![blueprint::InteractionSlot {
                        name:     "Gardener".into(),
                        capacity: 1,
                    }],
                    ..Default::default()
                },
            },
        ))
        .id();

    let small_tank = world
        .spawn((
            WorldObject,
            Name::new("FacilityTypeDef SmallTank"),
            graph::FacilityTypeDef {
                display_name: "Small tank".into(),
                volume:       120.0,
                sprite_id:    "facility/small-tank".into(),
                blueprint:    Blueprint {
                    fluid_storage: Some(blueprint::FluidStorage {
                        volume:         100.0,
                        optical_length: inverse_sphere_volume(100.0),
                    }),
                    ..Default::default()
                },
            },
        ))
        .id();

    StandardFacilityTypes { garden, small_tank }
}

struct StandardResidentAttrTypes {
    hp:          resident::attr::TypeId,
    weight:      resident::attr::TypeId,
    suffocation: resident::attr::TypeId,
}

fn gen_resident_attr_types(world: &mut World) -> StandardResidentAttrTypes {
    let hp = resident::attr::AddTypeCommand::new(resident::attr::TypeDef {
        name:          "HP".into(),
        default_value: 100.0,
        visibility:    enum_map! {
            view::SubscriptionLevel::Optical => false,
            view::SubscriptionLevel::Detail | view::SubscriptionLevel::Debug => true,
        },
    })
    .with_niche(resident::attr::Niche::Hitpoints)
    .run(world);
    let weight = resident::attr::AddTypeCommand::new(resident::attr::TypeDef {
        name:          "Weight".into(),
        default_value: 1.5, // we will just assume volume and weight are 1:1
        visibility:    enum_map! {
            view::SubscriptionLevel::Optical |
            view::SubscriptionLevel::Detail|
            view::SubscriptionLevel::Debug => true,
        },
    })
    .with_niche(resident::attr::Niche::Volume)
    .run(world);
    let suffocation = resident::attr::AddTypeCommand::new(resident::attr::TypeDef {
        name:          "Suffocation".into(),
        default_value: 0.0,
        visibility:    enum_map! {
            view::SubscriptionLevel::Optical | view::SubscriptionLevel::Detail => false,
            view::SubscriptionLevel::Debug => true,
        },
    })
    .run(world);
    StandardResidentAttrTypes { hp, weight, suffocation }
}

fn gen_resident_ambient_interactions(world: &mut World, std: &StandardTypes) {
    let mut interactions = world.resource_mut::<resident::ambient::Interactions>();
    interactions.list.push(
        resident::ambient::Interaction::new(
            "Breathing",
            const { duration_to_timesteps(Duration::from_secs(5)) },
        )
        .with_input(resident::ambient::Input::Fluid(reaction::input::Fluid {
            selector:       resident::ambient::AmbientFluidSelector,
            ty:             std.fluids.oxygen,
            max_rate:       fluid::Moles(0.05),
            conc_threshold: reaction::Threshold {
                curve:         reaction::Curve::Linear {
                    min_input:      0.1,
                    max_input:      0.8,
                    min_multiplier: 0.0,
                    max_multiplier: 1.0,
                },
                modifier_type: reaction::ThresholdModifierType::Multiplier,
            },
        }))
        .with_catalyst(resident::ambient::Catalyst::Pressure(reaction::catalyst::Pressure {
            selector:           resident::ambient::AmbientFluidSelector,
            pressure_threshold: reaction::Threshold {
                curve:         reaction::Curve::Linear {
                    min_input:      0.0,
                    max_input:      6.0,
                    min_multiplier: 0.0,
                    max_multiplier: 1.0,
                },
                modifier_type: reaction::ThresholdModifierType::Multiplier,
            },
        }))
        .with_catalyst(resident::ambient::Catalyst::ResidentAttr(
            reaction::catalyst::ResidentAttr {
                selector:   resident::ambient::SelfResidentSelector,
                attr:       std.attrs.suffocation,
                threshold:  reaction::Threshold {
                    curve:         reaction::Curve::Linear {
                        min_input:      0.0,
                        max_input:      1.0,
                        min_multiplier: 0.0,
                        max_multiplier: 1.0,
                    },
                    modifier_type: reaction::ThresholdModifierType::Maximum,
                },
                aggregator: reaction::Aggregator::Sum,
            },
        ))
        .with_output(resident::ambient::Output::Fluid(reaction::output::Fluid {
            selector: resident::ambient::AmbientFluidSelector,
            ty:       std.fluids.carbon_dioxide,
            max_rate: fluid::Moles(0.05),
        }))
        .with_output(resident::ambient::Output::ResidentAttr(
            reaction::output::ResidentAttr {
                selector: resident::ambient::SelfResidentSelector,
                attr:     std.attrs.suffocation,
                // this will be multiplied by efficiency, so it is actually not as high as it seems
                delta:    -3.0,
                min:      Some(0.0),
                max:      Some(1.0),
            },
        )),
    );
    interactions.list.push(
        resident::ambient::Interaction::new(
            "Respiration",
            const { duration_to_timesteps(Duration::from_secs(5)) },
        )
        .with_output(resident::ambient::Output::ResidentAttr(
            reaction::output::ResidentAttr {
                selector: resident::ambient::SelfResidentSelector,
                attr:     std.attrs.suffocation,
                delta:    0.1, // Reaches critical level after 50 seconds without breathing
                min:      Some(0.0),
                max:      Some(1.0),
            },
        )),
    );
    interactions.list.push(
        resident::ambient::Interaction::new(
            "Suffocation",
            const { duration_to_timesteps(Duration::from_secs(5)) },
        )
        .with_catalyst(resident::ambient::Catalyst::ResidentAttr(
            reaction::catalyst::ResidentAttr {
                selector:   resident::ambient::SelfResidentSelector,
                attr:       std.attrs.suffocation,
                aggregator: reaction::Aggregator::Sum,
                threshold:  reaction::Threshold {
                    curve:         reaction::Curve::Linear {
                        min_input:      0.95,
                        max_input:      1.0,
                        min_multiplier: 0.0,
                        max_multiplier: 1.0,
                    },
                    modifier_type: reaction::ThresholdModifierType::Maximum,
                },
            },
        ))
        .with_output(resident::ambient::Output::ResidentAttr(
            reaction::output::ResidentAttr {
                selector: resident::ambient::SelfResidentSelector,
                attr:     std.attrs.hp,
                delta:    -1.0, // Lose 1 HP every 5 seconds when suffocating, death in 8 minutes.
                min:      Some(0.0),
                max:      None,
            },
        )),
    );
}

fn gen_core(world: &mut World, std: &StandardTypes) -> CoreGen {
    let mut building = world.spawn((WorldObject,));
    building.reborrow_scope(|building| {
        building::SpawnCommand {
            name:           "Core".into(),
            position:       (0.0, 0.0).into(),
            radius:         15.0,
            wall_thickness: 0.8,
        }
        .apply(building);
    });

    let building_id = building.id();
    fill_atmosphere(&std.fluids, world, building_id);

    let mut tank = world.spawn(WorldObject);
    tank.reborrow_scope(|facility| {
        facility::SpawnCommand {
            name:             Some("Core water tank".into()),
            building:         building_id,
            ty:               std.facilities.small_tank,
            blueprint_params: blueprint::Params::default(),
        }
        .apply(facility);
    });
    let mut fluid_storage =
        tank.get_mut::<fluid::Storage>().expect("blueprint contains fluid storage");
    fluid_storage.set_fluid(std.fluids.water, fluid::Moles(80.0));
    tank.reborrow_scope(|facility| {
        fluid::SetTemperatureCommand { temperature: std.fluids.temperature }.apply(facility);
    });
    let tank = tank.id();

    CoreGen { building: building_id, tank }
}

struct CoreGen {
    building: Entity,
    tank:     Entity,
}

fn gen_garden(world: &mut World, std: &StandardTypes) -> GardenGen {
    let mut building = world.spawn((WorldObject,));
    building.reborrow_scope(|building| {
        building::SpawnCommand {
            name:           "Garden".into(),
            position:       (40.0, 0.0).into(),
            radius:         6.0,
            wall_thickness: STANDARD_WALL_THICKNESS,
        }
        .apply(building);
    });

    let building_id = building.id();
    fill_atmosphere(&std.fluids, world, building_id);

    let mut facility = world.spawn(WorldObject);
    facility.reborrow_scope(|facility| {
        facility::SpawnCommand {
            name:             None,
            building:         building_id,
            ty:               std.facilities.garden,
            blueprint_params: blueprint::Params {
                reactor: Some(blueprint::ReactorParams {
                    fluid_storages: [Some(building_id), None].into(),
                }),
            },
        }
        .apply(facility);
    });
    let facility = facility.id();

    GardenGen { building: building_id, facility }
}

struct GardenGen {
    building: Entity,
    facility: Entity,
}

fn spawn_corridor(
    world: &mut World,
    std: &StandardTypes,
    endpoints: AlphaBeta<Entity>,
    radius: f32,
    conduits: impl FnOnce(&mut World, Entity),
) {
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
            wall_thickness: STANDARD_WALL_THICKNESS,
        }
        .apply(corridor);
    });

    let corridor_id = corridor.id();
    fill_atmosphere(&std.fluids, world, corridor_id);

    spawn_edge(Alpha, world, endpoints, corridor_id);
    spawn_edge(Beta, world, endpoints, corridor_id);

    conduits(world, corridor_id);
}

fn spawn_fluid_pipe(
    world: &mut World,
    corridor: Entity,
    from_facility: Entity,
    to_facility: Entity,
    name: impl Into<String>,
    radius: f32,
) -> Entity {
    let mut pipe = world.spawn((WorldObject,));

    pipe.reborrow_scope(|pipe| {
        conduit::SpawnCommand {
            corridor,
            name: name.into(),
            radius,
            typed: conduit::TypedSpawn::FluidPipe,
        }
        .apply(pipe);
    });

    pipe.id()
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

fn connect_facility_pipe(world: &mut World, facility: Entity, pipe: Entity) {
    let connection = world.spawn((WorldObject,));

    connection::SpawnCommand { main: facility, peer: connection::SpawnPeer::Pipe(pipe) }
        .apply(connection);
}

fn spawn_resident_in_building(world: &mut World, building: Entity) {
    let mut resident = world.spawn((WorldObject,));
    resident.reborrow_scope(|resident| {
        resident::SpawnCommand {
            name: None,
            at:   resident::SpawnAt::Building { building, interior_pos: Vec3::ZERO },
        }
        .apply(resident);
    });
}

fn spawn_resident_in_facility_slot(world: &mut World, facility: Entity, slot_index: usize) {
    let mut resident = world.spawn((WorldObject,));
    resident.reborrow_scope(|resident| {
        resident::SpawnCommand {
            name: None,
            at:   resident::SpawnAt::Facility { facility, slot_index },
        }
        .apply(resident);
    });
}

fn fill_atmosphere(std: &StandardFluidTypes, world: &mut World, building: Entity) {
    let mut building = world.entity_mut(building);
    let mut storage =
        building.get_mut::<fluid::Storage>().expect("building must have fluid storage");
    for &(ty, fraction) in &std.atmosphere {
        let moles = fluid::Moles(fraction * storage.volume);
        storage.set_fluid(ty, moles);
    }

    fluid::SetTemperatureCommand { temperature: std.temperature }.apply(building);
}

fn inverse_sphere_volume(volume: f32) -> f32 { (volume * 3.0 / (4.0 * PI)).cbrt() }
