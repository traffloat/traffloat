use std::f32::consts::PI;

use bevy::ecs::entity::Entity;
use bevy::ecs::system::EntityCommand;
use bevy::ecs::world::World;

use crate::graph::{self, building, corridor};
use crate::util::AlphaBeta;
use crate::{WorldObject, fluid, view};

const STANDARD_WALL_THICKNESS: f32 = 0.5;

pub struct Config {}

/// Generate a basic physics world.
pub fn generate(world: &mut World, _: Config) {
    let facilities = gen_facility_types(world);
    let fluids = gen_fluid_types(world);
    let std = StandardTypes { facilities, fluids };
    let core = gen_core(world, &std);
    let garden = gen_garden(world, &std);
    gen_corridor(world, &std, AlphaBeta { alpha: core, beta: garden }, 1.1);
}

struct StandardTypes {
    facilities: StandardFacilityTypes,
    fluids:     StandardFluidTypes,
}

struct StandardFacilityTypes {
    core: Entity,
}

fn gen_facility_types(world: &mut World) -> StandardFacilityTypes {
    let core = world
        .spawn((WorldObject, graph::FacilityTypeDef { display: "Core".into(), volume: 5000.0 }))
        .id();
    let garden = world
        .spawn((WorldObject, graph::FacilityTypeDef { display: "Garden".into(), volume: 300.0 }))
        .id();
    StandardFacilityTypes { core }
}

struct StandardFluidTypes {
    filler:      fluid::TypeId,
    inhale:      fluid::TypeId,
    exhale:      fluid::TypeId,
    atmosphere:  Vec<(fluid::TypeId, f32)>,
    temperature: f32,
}

fn gen_fluid_types(world: &mut World) -> StandardFluidTypes {
    let mut types = world.resource_mut::<fluid::Types>();
    let filler = types.push(fluid::TypeDef {
        name:                 "Nitrogen".into(),
        molar_heat_capacity:  20.800,
        molar_density:        28.014,
        advective_fluidity:   0.4,
        diffusive_fluidity:   0.08,
        thermal_conductivity: 0.026,
        optical_extinction:   [1e-3, 1e-3, 1e-3],
    });
    let inhale = types.push(fluid::TypeDef {
        name:                 "Oxygen".into(),
        molar_heat_capacity:  21.000,
        molar_density:        31.998,
        advective_fluidity:   0.344,
        diffusive_fluidity:   0.0748,
        thermal_conductivity: 0.027,
        optical_extinction:   [1e-3, 1e-3, 1e-3],
    });
    let exhale = types.push(fluid::TypeDef {
        name:                 "Carbon dioxide".into(),
        molar_heat_capacity:  28.460,
        molar_density:        44.009,
        advective_fluidity:   0.475,
        diffusive_fluidity:   0.0635,
        thermal_conductivity: 0.017,
        optical_extinction:   [1e-3, 1e-3, 1e-3],
    });
    StandardFluidTypes {
        filler,
        inhale,
        exhale,
        atmosphere: [(filler, 0.78), (inhale, 0.21), (exhale, 0.01)].into(),
        temperature: 293.15,
    }
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
                ambient_volume: const { sphere_volume(15.0) - 5000.0 },
            },
        }
        .apply(building);
    });

    // TODO spawn facilities

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
                ambient_volume: const { sphere_volume(6.0) - 300.0 },
            },
        }
        .apply(building);
    });

    // TODO spawn facilities

    let building_id = building.id();
    std.fluids.fill_atmosphere(world, building_id);
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
            endpoints: endpoints.map(Some),
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
