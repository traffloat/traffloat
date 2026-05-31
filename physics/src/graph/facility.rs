use bevy::app::{self, App, Plugin};
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::name::Name;
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{EntityCommand, Query};
use bevy::ecs::world::EntityWorldMut;
use bevy::reflect::Reflect;
use traffloat_proto::proto;

use crate::graph::Building;
use crate::util::{QueryExt, WorldExt};
use crate::view;

pub mod blueprint;
pub use blueprint::Blueprint;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Facility>();
        app.register_type::<FacilityList>();
        app.register_type::<FacilityOf>();
        app.register_type::<FacilityTypeInstances>();
        app.register_type::<FacilityType>();

        app.add_systems(app::Update, init_viewer_system.in_set(view::SendUpdatesSystemSet::Init));
        app.add_systems(
            app::Update,
            (basic_incr_viewer_system, full_incr_viewer_system)
                .chain()
                .in_set(super::ViewSystemSets::Facility)
                .in_set(view::SendUpdatesSystemSet::Incr),
        );
    }
}

#[derive(Component, Reflect)]
pub struct Facility {
    pub name:   String,
    pub volume: f32,
}

/// Facilities in a building. Component on buildings.
#[derive(Component, Reflect)]
#[relationship_target(relationship = FacilityOf, linked_spawn)]
pub struct FacilityList(Vec<Entity>);

/// Building owning the facility. Component on facilities.
#[derive(Component, Reflect)]
#[relationship(relationship_target = FacilityList)]
pub struct FacilityOf(pub Entity);

/// Facility instances of a facility type. Component on facility types.
#[derive(Component, Reflect)]
#[relationship_target(relationship = FacilityType)]
pub struct FacilityTypeInstances(Vec<Entity>);

/// Facility type entity of a facility. Component on facilities.
#[derive(Component, Reflect)]
#[relationship(relationship_target = FacilityTypeInstances)]
pub struct FacilityType(pub Entity);

/// Describes a facility type. Component on facility types.
#[derive(Component)]
pub struct FacilityTypeDef {
    pub display_name: String,
    pub volume:       f32,
    pub sprite_id:    String,

    /// Blueprint for constructing this facility type.
    pub blueprint: Blueprint,
}

/// Spawns the facility and recomputes building ambient volume.
///
/// Facility blueprint is not handled by this command since they require additional parameters.
pub struct SpawnCommand {
    pub name:     Option<String>,
    pub building: Entity,
    pub ty:       Entity,
}

impl EntityCommand for SpawnCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        let Some(typedef) = entity.world().log_get::<FacilityTypeDef>(self.ty) else { return };
        let volume = typedef.volume;
        let name = self.name.clone().unwrap_or_else(|| typedef.display_name.clone());

        entity.insert((
            Name::new("Facility"),
            FacilityType(self.ty),
            FacilityOf(self.building),
            Facility { name, volume },
        ));
        entity.reborrow_scope(|entity| view::AddViewableCommand.apply(entity));

        // TODO recompute remaining ambient volume in building
    }
}

fn init_viewer_system(
    facility_query: Query<(&Facility, &view::Viewable, &FacilityType, &FacilityOf)>,
    building_query: Query<&view::Viewable, With<Building>>,
    type_query: Query<&FacilityTypeDef>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    for (facility, viewable, &FacilityType(ty), &FacilityOf(building_entity)) in facility_query {
        messages.write_batch(viewable.broadcast_new(|| {
            let building_viewable = building_query.log_get(building_entity)?;
            let typedef = type_query.log_get(ty)?;
            Some(proto::Update::NewFacility(proto::NewFacility {
                id:       viewable.id,
                building: building_viewable.id,
                name:     facility.name.clone(),
                volume:   facility.volume,
                display:  proto::FacilityDisplay {
                    sprite_id: typedef.sprite_id.clone(),
                    taint:     Color::WHITE.into(),
                },
            }))
        }));
    }
}

fn basic_incr_viewer_system() {
    // TODO
}

fn full_incr_viewer_system() {
    // TODO
}
