use std::f32::consts::PI;

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::name::Name;
use bevy::ecs::query::With;
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{EntityCommand, Query};
use bevy::ecs::world::EntityWorldMut;
use bevy::reflect::Reflect;
use traffloat_proto::proto;

use crate::graph::conduit::ListOnCorridor;
use crate::graph::{Conduit, ViewInitSystemSets};
use crate::util::{AlphaBeta, EntityWorldMutExt, WorldExt};
use crate::{Vector, fluid, view};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<NextCorridorId>();
        app.register_type::<Corridor>();

        app.init_resource::<NextCorridorId>();
        app.add_systems(
            app::Update,
            init_viewer_system
                .in_set(view::SendUpdatesSystemSet::Init)
                .in_set(ViewInitSystemSets::Corridor),
        );
        app.add_systems(
            app::Update,
            incr_viewer_system
                .in_set(super::ViewIncrSystemSets::Corridor)
                .in_set(view::SendUpdatesSystemSet::Incr),
        );
    }
}

#[derive(Component, Reflect)]
pub struct Corridor {
    pub name:               String,
    pub length:             f32,
    pub radius:             f32,
    pub wall_thickness:     f32,
    pub ambient_area:       f32,
    pub endpoint_positions: AlphaBeta<Vector>,
}

#[derive(Resource, Default, Reflect)]
struct NextCorridorId(u64);

pub struct SpawnCommand {
    pub name:               Option<String>,
    pub endpoint_positions: AlphaBeta<Vector>,
    pub length:             f32,
    pub radius:             f32,
    pub wall_thickness:     f32,
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
        let ambient_area = self.radius * self.radius * PI;
        entity.insert((
            Name::new(format!("Corridor {name}")),
            Corridor {
                name,
                length: self.length,
                radius: self.radius,
                wall_thickness: self.wall_thickness,
                ambient_area,
                endpoint_positions: self.endpoint_positions,
            },
        ));
        entity.reborrow_scope(|entity| view::AddViewableCommand.apply(entity));

        // ambient conduit
        entity.reborrow_scope(|entity| {
            fluid::AddStorageCommand {
                volume:         ambient_area * self.length,
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
        let used_by_conduits: f32 = if let Some(conduit_list) = entity.get::<ListOnCorridor>() {
            let conduits: Vec<_> = conduit_list.iter().collect();
            conduits
                .iter()
                .filter_map(|&f| entity.world().log_get::<Conduit>(f))
                .map(|f| f.radius * f.radius)
                .sum()
        } else {
            0.0
        };
        let Some(mut corridor) = entity.log_get_mut::<Corridor>() else { return };

        let ambient_base = corridor.radius * corridor.radius - used_by_conduits;
        let ambient_area = ambient_base * PI;
        corridor.ambient_area = ambient_area;
        let ambient_volume = ambient_area * corridor.length;

        if let Some(mut fluid) = entity.log_get_mut::<fluid::Storage>() {
            fluid.volume = ambient_volume;
        }
    }
}

fn init_viewer_system(
    corridor_query: Query<(&Corridor, &view::Viewable)>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    for (corridor, viewable) in corridor_query {
        messages.write_batch(viewable.broadcast_new(|| {
            [proto::Update::NewCorridor(proto::NewCorridor {
                id:             viewable.id,
                name:           corridor.name.clone(),
                alpha_position: corridor.endpoint_positions.alpha,
                beta_position:  corridor.endpoint_positions.beta,
                radius:         corridor.radius,
                wall_thickness: corridor.wall_thickness,
            })]
        }));
    }
}

fn incr_viewer_system(
    mut throttle: view::BroadcastThrottle,
    corridor_query: Query<(&view::Viewable, &fluid::Storage), With<Corridor>>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    if !throttle.should_run() {
        return;
    }

    for (viewable, storage) in corridor_query.iter() {
        messages.write_batch(viewable.broadcast_update(|level| match level {
            view::SubscriptionLevel::Basic => {
                [proto::Update::UpdateCorridor(proto::UpdateCorridor {
                    id:    viewable.id,
                    color: proto::Color(storage.rgba),
                })]
            }
            view::SubscriptionLevel::Full => {
                [proto::Update::UpdateCorridorFull(proto::UpdateCorridorFull {
                    id:            viewable.id,
                    color:         proto::Color(storage.rgba),
                    ambient_fluid: storage.to_proto(),
                })]
            }
        }));
    }
}
