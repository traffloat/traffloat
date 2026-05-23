use bevy::ecs::entity::Entity;
use bevy::ecs::system::EntityCommand;
use bevy::ecs::world::World;

use crate::graph::{self, building};
use crate::{WorldObject, fluid, view};

pub struct Config {}

/// Generate a basic physics world.
pub fn generate(world: &mut World, _: Config) {
    let facilities = gen_facility_types(world);
    let fluids = gen_fluid_types(world);
    let std = StandardTypes { facilities, fluids };
    gen_core(world, &std);
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
        .spawn((WorldObject, graph::FacilityTypeDef { display: "Core".into(), volume: 100.0 }))
        .id();
    StandardFacilityTypes { core }
}

struct StandardFluidTypes {
    filler: fluid::TypeId,
}

fn gen_fluid_types(world: &mut World) -> StandardFluidTypes {
    let mut types = world.resource_mut::<fluid::Types>();
    let filler = types.push(fluid::TypeDef {
        name:                 "Nitrogen".into(),
        molar_heat_capacity:  1040.0,
        molar_density:        14.0,
        advective_fluidity:   0.1,
        diffusive_fluidity:   0.01,
        thermal_conductivity: 0.01,
        optical_extinction:   [1e-3, 1e-3, 1e-3],
    });
    StandardFluidTypes { filler }
}

fn gen_core(world: &mut World, std: &StandardTypes) {
    let mut building = world.spawn((WorldObject,));
    building.reborrow_scope(|building| {
        building::SpawnCommand {
            building: graph::Building {
                position:       (0.0, 0.0).into(),
                radius:         20.0,
                wall_thickness: 0.8,
                ambient_volume: 500.0,
            },
        }
        .apply(building);
    });
    let mut fluid = building
        .get_mut::<fluid::Storage>()
        .expect("building spawn includes ambient fluid storage");
    fluid.set_fluid(std.fluids.filler, fluid::Moles(10.0));
}
