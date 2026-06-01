use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::name::Name;
use bevy::ecs::query::With;
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, EntityCommand, Query};
use bevy::ecs::world::EntityWorldMut;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};
use traffloat_proto::proto;

use crate::graph::{Facility, FacilityList};
use crate::util::{AlphaBeta, EntityWorldMutExt, WorldExt};
use crate::{Vector, fluid, view};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Building>();

        app.add_systems(app::Update, init_viewer_system.in_set(view::SendUpdatesSystemSet::Init));
        app.add_systems(
            app::Update,
            incr_viewer_system
                .in_set(super::ViewSystemSets::Building)
                .in_set(view::SendUpdatesSystemSet::Incr),
        );
    }
}

#[derive(Component, Reflect)]
pub struct Building {
    pub name:           String,
    pub position:       Vector,
    pub radius:         f32,
    pub wall_thickness: f32,
    pub ambient_volume: f32,
}

pub struct SpawnCommand {
    pub name:           String,
    pub position:       Vector,
    pub radius:         f32,
    pub wall_thickness: f32,
}

impl EntityCommand for SpawnCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        let ambient_volume = sphere_volume(self.radius);

        entity.insert((
            Name::new(format!("Building {}", self.name)),
            Building {
                name: self.name,
                position: self.position,
                radius: self.radius,
                wall_thickness: self.wall_thickness,
                ambient_volume,
            },
        ));
        entity.reborrow_scope(|entity| view::AddViewableCommand.apply(entity));

        // ambient storage
        entity.reborrow_scope(|entity| {
            fluid::AddStorageCommand {
                volume:         ambient_volume,
                optical_length: self.radius,
            }
            .apply(entity);
        });
    }
}

pub struct DespawnCommand;

impl EntityCommand for DespawnCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        view::on_viewable_despawn(&mut entity);
        entity.despawn();
    }
}

pub struct RecomputeAmbientVolume;

impl EntityCommand for RecomputeAmbientVolume {
    fn apply(self, mut entity: EntityWorldMut) {
        let used_by_facilities: f32 = if let Some(facility_list) = entity.get::<FacilityList>() {
            let facilities: Vec<_> = facility_list.iter().collect();
            facilities
                .iter()
                .filter_map(|&f| entity.world().log_get::<Facility>(f))
                .map(|f| f.volume)
                .sum()
        } else {
            0.0
        };
        let Some(mut building) = entity.log_get_mut::<Building>() else { return };

        let ambient_volume = sphere_volume(building.radius) - used_by_facilities;
        building.ambient_volume = ambient_volume;

        if let Some(mut fluid) = entity.log_get_mut::<fluid::Storage>() {
            fluid.volume = ambient_volume;
        }
    }
}

fn sphere_volume(radius: f32) -> f32 { radius.powi(3) * std::f32::consts::PI * 4.0 / 3.0 }

fn init_viewer_system(
    building_query: Query<(&Building, &view::Viewable)>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    for (building, viewable) in building_query {
        messages.write_batch(viewable.broadcast_new(|| {
            [proto::Update::NewBuilding(proto::NewBuilding {
                id:             viewable.id,
                name:           building.name.clone(),
                position:       building.position,
                radius:         building.radius,
                wall_thickness: building.wall_thickness,
            })]
        }));
    }
}

fn incr_viewer_system(
    mut throttle: view::BroadcastThrottle,
    building_query: Query<(&view::Viewable, &fluid::Storage), With<Building>>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    if !throttle.should_run() {
        return;
    }

    for (viewable, storage) in building_query {
        messages.write_batch(viewable.broadcast_update(|level| match level {
            view::SubscriptionLevel::Basic => {
                [proto::Update::UpdateBuilding(proto::UpdateBuilding {
                    id:    viewable.id,
                    color: proto::Color(storage.rgba),
                })]
            }
            view::SubscriptionLevel::Full => {
                [proto::Update::UpdateBuildingFull(proto::UpdateBuildingFull {
                    id:            viewable.id,
                    color:         proto::Color(storage.rgba),
                    ambient_fluid: proto::FluidStorageFull {
                        volume:      storage.volume,
                        pressure:    storage.pressure,
                        temperature: storage.temperature,
                        types:       storage.types.iter().map(|typed| typed.moles.0).collect(),
                    },
                })]
            }
        }));
    }
}
