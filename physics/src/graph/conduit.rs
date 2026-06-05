use std::f32::consts::PI;

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::name::Name;
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{EntityCommand, Query};
use bevy::ecs::world::EntityWorldMut;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};
use traffloat_proto::proto;

use crate::graph::{Corridor, corridor};
use crate::util::{AlphaBeta, QueryExt};
use crate::{Vector, fluid, view};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Conduit>();
        app.register_type::<ConduitOf>();
        app.register_type::<ConduitList>();

        app.add_systems(app::Update, init_viewer_system.in_set(view::SendUpdatesSystemSet::Init));
        app.add_systems(
            app::Update,
            incr_viewer_system
                .in_set(super::ViewSystemSets::Facility)
                .in_set(view::SendUpdatesSystemSet::Incr),
        );
    }
}

#[derive(Component, Reflect)]
pub struct Conduit {
    pub name:   String,
    pub radius: f32,
    pub ty:     ConduitType,
}

/// Conduits in a corridor. Component on corridors.
#[derive(Component, Reflect)]
#[relationship_target(relationship = ConduitOf, linked_spawn)]
pub struct ConduitList(Vec<Entity>);

/// Corridor owning the conduit. Component on conduits.
#[derive(Component, Reflect)]
#[relationship(relationship_target = ConduitList)]
pub struct ConduitOf(pub Entity);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum ConduitType {
    FluidPipe,
    // PowerCable,
    // VehicleRail,
}

pub struct SpawnCommand {
    pub corridor: Entity,
    pub name:     String,
    pub radius:   f32,
    pub typed:    TypedSpawn,
}

pub enum TypedSpawn {
    FluidPipe,
}

impl EntityCommand for SpawnCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        let Some(corridor_length) =
            entity.world().get::<Corridor>(self.corridor).map(|corridor| corridor.length)
        else {
            return;
        };

        entity.insert((
            Name::new("Conduit"),
            Conduit {
                name:   self.name,
                radius: self.radius,
                ty:     match self.typed {
                    TypedSpawn::FluidPipe => ConduitType::FluidPipe,
                },
            },
            ConduitOf(self.corridor),
        ));
        entity.reborrow_scope(|entity| view::AddViewableCommand.apply(entity));

        entity.world_scope(|world| {
            corridor::RecomputeAmbientVolume.apply(world.entity_mut(self.corridor));
        });

        match self.typed {
            TypedSpawn::FluidPipe => entity.reborrow_scope(|entity| {
                fluid::AddStorageCommand {
                    volume:         self.radius * self.radius * PI * corridor_length,
                    optical_length: self.radius,
                }
                .apply(entity);
            }),
        }
    }
}

fn init_viewer_system(
    conduit_query: Query<(&Conduit, &view::Viewable, &ConduitOf)>,
    corridor_query: Query<&view::Viewable, With<Corridor>>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    for (conduit, viewable, &ConduitOf(corridor_entity)) in conduit_query {
        messages.write_batch(viewable.broadcast_new(|| {
            let corridor_viewable = corridor_query.log_get(corridor_entity)?;
            Some(proto::Update::NewConduit(proto::NewConduit {
                id:       viewable.id,
                name:     conduit.name.clone(),
                corridor: corridor_viewable.id,
                radius:   conduit.radius,
                ty:       match conduit.ty {
                    ConduitType::FluidPipe => proto::ConduitType::FluidPipe,
                },
            }))
        }));
    }
}

fn incr_viewer_system(
    mut throttle: view::BroadcastThrottle,
    conduit_query: Query<(&view::Viewable, Option<&fluid::Storage>), With<Conduit>>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    if !throttle.should_run() {
        return;
    }

    for (viewable, fluid_storage) in conduit_query {
        if let Some(fluid_storage) = fluid_storage {
            let color = proto::Color(fluid_storage.rgba);
            messages.write_batch(viewable.broadcast_update(|level| {
                match level {
                    view::SubscriptionLevel::Basic => {
                        [proto::UpdateFluidConduit { id: viewable.id, color }.into()]
                    }
                    view::SubscriptionLevel::Full => [proto::UpdateFluidConduitFull {
                        id: viewable.id,
                        color,
                        fluid: fluid_storage.to_proto(),
                    }
                    .into()],
                }
            }));
        }
    }
}
