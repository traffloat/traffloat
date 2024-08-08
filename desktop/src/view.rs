use bevy::app::{self, App};
use bevy::core_pipeline::core_3d::Camera3dBundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Query};
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::state::state;
use bevy::transform::components::Transform;
use traffloat_view::viewer;

use crate::AppState;

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(state::OnEnter(AppState::GameView), setup);
        app.add_systems(state::OnExit(AppState::GameView), teardown);
    }
}

#[derive(Component)]
struct Owned;

fn setup(mut commands: Commands) {
    commands.spawn((Owned, Camera3dBundle::default()));
    commands.spawn((
        Owned,
        viewer::Bundle::builder()
            .position(Transform::default())
            .range(viewer::Range { distance: 100. })
            .build(),
    ));
}

fn teardown(mut commands: Commands, query: Query<Entity, With<Owned>>) {
    query.into_iter().for_each(|entity| {
        commands.entity(entity).despawn_recursive();
    });
}
