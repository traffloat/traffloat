use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, EntityCommand, Query};
use bevy::ecs::world::EntityWorldMut;
use serde::{Deserialize, Serialize};
use traffloat_proto::proto;

use crate::util::AlphaBeta;
use crate::{Vector, fluid, view};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.add_systems(app::Update, init_viewer_system.in_set(view::SendUpdatesSystemSet::Init));
        app.add_systems(
            app::Update,
            (basic_incr_viewer_system, full_incr_viewer_system)
                .chain()
                .in_set(view::SendUpdatesSystemSet::Incr),
        );
    }
}

#[derive(Component)]
pub struct Building {
    pub position:       Vector,
    pub radius:         f32,
    pub wall_thickness: f32,
    pub ambient_volume: f32,
}

pub struct SpawnCommand {
    pub building: Building,
}

impl EntityCommand for SpawnCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        let ambient_volume = self.building.ambient_volume;
        let radius = self.building.radius;

        entity.insert(self.building);
        entity.reborrow_scope(|entity| view::AddViewableCommand.apply(entity));

        // ambient storage
        entity.reborrow_scope(|entity| {
            fluid::AddStorageCommand { ambient_volume, radius }.apply(entity)
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

fn init_viewer_system(
    building_query: Query<(&Building, &view::Viewable)>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    for (building, viewable) in building_query.iter() {
        if !viewable.new_subscribers.is_empty() {
            messages.write(view::SentUpdate {
                viewers: viewable.new_subscribers.iter().copied().collect(),
                body:    proto::Update::NewBuilding(proto::NewBuilding {
                    id:             viewable.id,
                    position:       building.position,
                    radius:         building.radius,
                    wall_thickness: building.wall_thickness,
                }),
            });
        }
    }
}

fn basic_incr_viewer_system(
    building_query: Query<(&Building, &view::Viewable, &fluid::Storage)>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    for (building, viewable, storage) in building_query.iter() {
        let subs = &viewable.subscribers[view::SubscriptionLevel::Basic];
        if !subs.is_empty() {
            messages.write(view::SentUpdate {
                viewers: subs.iter().copied().collect(),
                body:    proto::Update::UpdateBuilding(proto::UpdateBuilding {
                    id:    viewable.id,
                    color: proto::Color(storage.rgba),
                }),
            });
        }
    }
}

fn full_incr_viewer_system(
    building_query: Query<(&Building, &view::Viewable, &fluid::Storage)>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    for (building, viewable, storage) in building_query.iter() {
        let subs = &viewable.subscribers[view::SubscriptionLevel::Full];
        if !subs.is_empty() {
            messages.write(view::SentUpdate {
                viewers: subs.iter().copied().collect(),
                body:    proto::Update::UpdateBuildingFull(proto::UpdateBuildingFull {
                    id:      viewable.id,
                    color:   proto::Color(storage.rgba),
                    ambient: proto::FluidStorageFull {
                        pressure:    storage.pressure,
                        temperature: storage.temperature,
                        fluids:      storage.types.iter().map(|typed| typed.moles.0).collect(),
                    },
                }),
            });
        }
    }
}
