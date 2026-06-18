use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::ecs::name::Name;
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::Single;
use bevy::ecs::world::World;
use bevy::state::condition::in_state;
use bevy::state::state::NextState;
use traffloat_physics::{request, view};

use crate::scene::{InboundUpdate, LevelState, OutboundRequest};

pub(super) struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.add_systems(
            app::Update,
            sync_outbound_system.run_if(in_state(LevelState::Singleplayer)),
        );
        app.add_systems(
            app::Update,
            sync_inbound_system.run_if(in_state(LevelState::Singleplayer)),
        );
    }
}

/// Marks the viewer entity for singleplayer client.
#[derive(Component)]
struct SinglePlayerViewer;
pub fn setup(world: &mut World) {
    world.resource_mut::<NextState<LevelState>>().set(LevelState::Singleplayer);
    world.spawn((
        Name::new("SinglePlayerViewer"),
        traffloat_physics::WorldObject,
        SinglePlayerViewer,
        view::Viewer::default(),
    ));
}

fn sync_outbound_system(
    mut reader: MessageReader<OutboundRequest>,
    mut writer: MessageWriter<request::Approved>,
    viewer: Single<Entity, With<SinglePlayerViewer>>,
) {
    writer.write_batch(
        reader
            .read()
            .map(|request| request::Approved { viewer: *viewer, body: request.body.clone() }),
    );
}

fn sync_inbound_system(
    mut reader: MessageReader<view::SentUpdate>,
    mut writer: MessageWriter<InboundUpdate>,
    viewer: Single<Entity, With<SinglePlayerViewer>>,
) {
    let viewer = *viewer;
    writer.write_batch(
        reader
            .read()
            .filter(|update| update.viewers.contains(&viewer))
            .map(|update| InboundUpdate { body: update.body.clone() }),
    );
}
