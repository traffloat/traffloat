use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::name::Name;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{EntityCommand, Query};
use bevy::ecs::world::EntityWorldMut;
use traffloat_proto::proto;

use crate::util::AlphaBeta;
use crate::{Vector, fluid, view};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextCorridorId>();
        app.add_systems(app::Update, init_viewer_system.in_set(view::SendUpdatesSystemSet::Init));
        app.add_systems(
            app::Update,
            (basic_incr_viewer_system, full_incr_viewer_system)
                .chain()
                .in_set(super::ViewSystemSets::Corridor)
                .in_set(view::SendUpdatesSystemSet::Incr),
        );
    }
}

#[derive(Component)]
pub struct Corridor {
    pub name:               String,
    pub length:             f32,
    pub radius:             f32,
    pub wall_thickness:     f32,
    pub ambient_area:       f32,
    pub endpoint_positions: AlphaBeta<Vector>,
}

#[derive(Resource, Default)]
struct NextCorridorId(u64);

pub struct SpawnCommand {
    pub name:               Option<String>,
    pub endpoint_positions: AlphaBeta<Vector>,
    pub length:             f32,
    pub radius:             f32,
    pub wall_thickness:     f32,
    pub ambient_area:       f32,
}

impl EntityCommand for SpawnCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        let name = self.name.unwrap_or_else(|| {
            entity.world_scope(|world| {
                let mut next = world.resource_mut::<NextCorridorId>();
                let out = next.0;
                next.0 += 1;
                format!("#{out}")
            })
        });
        entity.insert((
            Name::new(format!("Corridor {name}")),
            Corridor {
                name,
                length: self.length,
                radius: self.radius,
                wall_thickness: self.wall_thickness,
                ambient_area: self.ambient_area,
                endpoint_positions: self.endpoint_positions,
            },
        ));
        entity.reborrow_scope(|entity| view::AddViewableCommand.apply(entity));

        // ambient conduit
        entity.reborrow_scope(|entity| {
            fluid::AddStorageCommand {
                ambient_volume: self.ambient_area * self.length,
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

fn init_viewer_system(
    corridor_query: Query<(&Corridor, &view::Viewable)>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    for (corridor, viewable) in corridor_query {
        if !viewable.new_subscribers.is_empty() {
            messages.write(view::SentUpdate {
                viewers: viewable.new_subscribers.iter().copied().collect(),
                body:    proto::Update::NewCorridor(proto::NewCorridor {
                    id:             viewable.id,
                    name:           corridor.name.clone(),
                    alpha_position: corridor.endpoint_positions.alpha,
                    beta_position:  corridor.endpoint_positions.beta,
                    radius:         corridor.radius,
                    wall_thickness: corridor.wall_thickness,
                }),
            });
        }
    }
}

fn basic_incr_viewer_system(
    mut throttle: view::BroadcastThrottle,
    corridor_query: Query<(&Corridor, &view::Viewable, &fluid::Storage)>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    if !throttle.should_run() {
        return;
    }

    for (building, viewable, storage) in corridor_query {
        let subs = &viewable.subscribers[view::SubscriptionLevel::Basic];
        if !subs.is_empty() {
            messages.write(view::SentUpdate {
                viewers: subs.iter().copied().collect(),
                body:    proto::Update::UpdateCorridor(proto::UpdateCorridor {
                    id:    viewable.id,
                    color: proto::Color(storage.rgba),
                }),
            });
        }
    }
}

fn full_incr_viewer_system(
    mut throttle: view::BroadcastThrottle,
    corridor_query: Query<(&Corridor, &view::Viewable, &fluid::Storage)>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    if !throttle.should_run() {
        return;
    }

    for (corridor, viewable, storage) in corridor_query.iter() {
        let subs = &viewable.subscribers[view::SubscriptionLevel::Full];
        if !subs.is_empty() {
            messages.write(view::SentUpdate {
                viewers: subs.iter().copied().collect(),
                body:    proto::Update::UpdateCorridorFull(proto::UpdateCorridorFull {
                    id:            viewable.id,
                    color:         proto::Color(storage.rgba),
                    ambient_fluid: proto::FluidStorageFull {
                        volume:      storage.volume,
                        pressure:    storage.pressure,
                        temperature: storage.temperature,
                        types:       storage.types.iter().map(|typed| typed.moles.0).collect(),
                    },
                }),
            });
        }
    }
}
