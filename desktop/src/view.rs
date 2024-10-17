use bevy::app::{self, App};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::schedule::SystemSet;
use bevy::ecs::system::{Commands, Query, Res, ResMut};
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::state::state;
use bevy::transform::components::Transform;
use bevy::winit::WinitSettings;
use traffloat_view::viewer;

use crate::AppState;

// mod background;
mod camera;
mod delegate;
mod diagnostics;
mod object;

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((diagnostics::Plugin, camera::Plugin, object::Plugin));

        app.add_systems(state::OnEnter(AppState::GameView), setup_singleplayer_server);
        app.add_systems(state::OnEnter(AppState::GameView), setup_view);
        app.add_systems(state::OnExit(AppState::GameView), teardown);
    }
}

#[derive(Component)]
struct Owned;

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
struct InputSystemSet;

fn setup_singleplayer_server(mut commands: Commands, viewer_ids: Res<viewer::SidIndex>) {
    commands.spawn((
        Owned,
        viewer::Bundle::builder()
            .id(viewer_ids.next_id())
            .position(Transform::default())
            .range(viewer::Range { distance: 100. })
            .build(),
    ));
}

fn setup_view(mut winit_settings: ResMut<WinitSettings>) {
    *winit_settings = WinitSettings::game();
}

fn teardown(mut commands: Commands, owned_query: Query<Entity, With<Owned>>) {
    owned_query.into_iter().for_each(|entity| {
        commands.entity(entity).despawn_recursive();
    });
}
