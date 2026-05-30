use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;

pub mod blueprint;
use bevy::ecs::name::Name;
use bevy::ecs::system::EntityCommand;
use bevy::ecs::world::EntityWorldMut;
pub use blueprint::Blueprint;

use crate::util::WorldExt;

#[derive(Component)]
pub struct Facility {
    pub volume: f32,
}

/// Facilities in a building. Component on buildings.
#[derive(Component)]
#[relationship_target(relationship = FacilityOf, linked_spawn)]
pub struct FacilityList(Vec<Entity>);

/// Building owning the facility. Component on facilities.
#[derive(Component)]
#[relationship(relationship_target = FacilityList)]
pub struct FacilityOf(pub Entity);

/// Facility instances of a facility type. Component on facility types.
#[derive(Component)]
#[relationship_target(relationship = FacilityType)]
pub struct FacilityTypeInstances(Vec<Entity>);

/// Facility type entity of a facility. Component on facilities.
#[derive(Component)]
#[relationship(relationship_target = FacilityTypeInstances)]
pub struct FacilityType(pub Entity);

/// Describes a facility type. Component on facility types.
#[derive(Component)]
pub struct FacilityTypeDef {
    pub display: String,
    pub volume:  f32,

    /// Blueprint for constructing this facility type.
    pub blueprint: Blueprint,
}

/// Spawns the facility and recomputes building ambient volume.
///
/// Facility blueprint is not handled by this command since they require additional parameters.
pub struct SpawnCommand {
    pub building: Entity,
    pub ty:       Entity,
}

impl EntityCommand for SpawnCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        let Some(typedef) = entity.world().log_get::<FacilityTypeDef>(self.ty) else { return };
        let volume = typedef.volume;

        entity.insert((
            Name::new("Facility"),
            FacilityType(self.ty),
            FacilityOf(self.building),
            Facility { volume },
        ));
    }
}
