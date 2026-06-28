use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::name::Name;
use bevy::ecs::query::{QueryData, With};
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{EntityCommand, Query};
use bevy::ecs::world::EntityWorldMut;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};
use traffloat_proto::proto;

use crate::graph::{Building, ViewInitSystemSets, building};
use crate::persist::AppExt;
use crate::util::{QueryExt, WorldExt};
use crate::{fluid, view};

pub mod blueprint;
pub use blueprint::Blueprint;

mod persist;
pub use persist::Persist;
mod persist_types;
pub use persist_types::Persist as PersistTypes;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Facility>();
        app.register_type::<ListOnBuilding>();
        app.register_type::<OfBuilding>();
        app.register_type::<FacilityTypeInstances>();
        app.register_type::<FacilityType>();
        app.register_type::<FacilityTypeDef>();

        app.register_persistable(Persist);
        app.register_persistable(PersistTypes);

        app.add_systems(
            app::Update,
            init_viewer_system
                .in_set(view::SendUpdatesSystemSet::Init)
                .in_set(ViewInitSystemSets::Facility),
        );
        app.add_systems(
            app::Update,
            (incr_viewer_system)
                .in_set(super::ViewIncrSystemSets::Facility)
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
#[relationship_target(relationship = OfBuilding, linked_spawn)]
pub struct ListOnBuilding(Vec<Entity>);

/// Building owning the facility. Component on facilities.
#[derive(Component, Reflect)]
#[relationship(relationship_target = ListOnBuilding)]
pub struct OfBuilding(pub Entity);

/// Facility instances of a facility type. Component on facility types.
#[derive(Component, Reflect)]
#[relationship_target(relationship = FacilityType)]
pub struct FacilityTypeInstances(Vec<Entity>);

/// Facility type entity of a facility. Component on facilities.
#[derive(Component, Reflect)]
#[relationship(relationship_target = FacilityTypeInstances)]
pub struct FacilityType(pub Entity);

/// Describes a facility type. Component on facility types.
#[derive(Debug, Clone, Serialize, Deserialize, Component, Reflect)]
pub struct FacilityTypeDef {
    pub display_name: String,
    pub volume:       f32,
    pub sprite_id:    String,

    /// Blueprint for constructing this facility type.
    #[reflect(ignore, default)]
    pub blueprint: Blueprint,
}

/// Spawns the facility and recomputes building ambient volume.
///
/// Facility blueprint is not handled by this command since they require additional parameters.
pub struct SpawnCommand {
    pub name:     Option<String>,
    pub building: Entity,
    pub ty:       Entity,

    pub blueprint_params: blueprint::Params,
}

impl EntityCommand for SpawnCommand {
    type Out = ();
    fn apply(self, mut entity: EntityWorldMut) {
        let Some(typedef) = entity.world().log_get::<FacilityTypeDef>(self.ty) else { return };
        let volume = typedef.volume;
        let name = self.name.clone().unwrap_or_else(|| typedef.display_name.clone());

        // This is deliberately different from the building culling rect.
        // Buildings need to be subscribed whenever any connections are spawned,
        // On the other hand, facilities only need to be subscribed
        // when the building itself is directly visible.
        let building_rect = entity
            .world()
            .log_get::<Building>(self.building)
            .map(|b| view::CullingRect(b.base_rect()))
            .unwrap_or_default();

        entity.insert((
            Name::new("Facility"),
            FacilityType(self.ty),
            OfBuilding(self.building),
            Facility { name, volume },
            building_rect,
        ));
        entity.reborrow_scope(|entity| view::AddViewableCommand.apply(entity));

        entity.world_scope(|world| {
            building::RecomputeAmbientVolume.apply(world.entity_mut(self.building));
        });

        let Some(def) = entity.world().log_get::<FacilityTypeDef>(self.ty) else { return };
        def.blueprint.populate(&self.blueprint_params)(&mut entity);
    }
}

fn init_viewer_system(
    facility_query: Query<(
        &Facility,
        &view::Viewable,
        &FacilityType,
        &OfBuilding,
        Option<&fluid::Storage>,
    )>,
    building_query: Query<&view::Viewable, With<Building>>,
    type_query: Query<&FacilityTypeDef>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    for (facility, viewable, &FacilityType(ty), &OfBuilding(building_entity), fluid_storage) in
        facility_query
    {
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
                    taint:     fluid_storage.map(|s| proto::Color(s.rgba)),
                },
            }))
        }));
    }
}

fn incr_viewer_system(
    mut throttle: view::BroadcastThrottle,
    facility_query: Query<IncrData, IncrFilter>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    if !throttle.should_run() {
        return;
    }

    for facility in facility_query {
        if let Some((fluid_storage, sensor)) = facility.storage {
            let color = proto::Color(fluid_storage.rgba);
            messages.write_batch(facility.viewable.broadcast_update(|level| {
                let taint_update = proto::Update::UpdateFacilityTaint(proto::UpdateFacilityTaint {
                    id:    facility.viewable.id,
                    taint: color,
                });
                let fluid_update = match level {
                    view::SubscriptionLevel::Optical => None,
                    view::SubscriptionLevel::Detail => Some(
                        proto::UpdateFacilityFluid {
                            id:    facility.viewable.id,
                            fluid: fluid_storage.to_proto_normal(sensor),
                        }
                        .into(),
                    ),
                    view::SubscriptionLevel::Debug => Some(
                        proto::UpdateFacilityFluid {
                            id:    facility.viewable.id,
                            fluid: fluid_storage.to_proto_debug(),
                        }
                        .into(),
                    ),
                };
                [Some(taint_update), fluid_update].into_iter().flatten()
            }));
        }
    }
}

#[derive(QueryData)]
struct IncrData {
    viewable: &'static view::Viewable,
    storage:  Option<(&'static fluid::Storage, &'static fluid::Sensor)>,
}

type IncrFilter = With<Facility>;
