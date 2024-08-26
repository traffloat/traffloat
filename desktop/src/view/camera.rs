use bevy::app::{self, App};
use bevy::core_pipeline::core_3d::{Camera3d, Camera3dBundle};
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Query, Res};
use bevy::math::Vec3;
use bevy::state::condition::in_state;
use bevy::state::state;
use bevy::time::Time;
use bevy::transform::components::Transform;

use crate::AppState;

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(state::OnEnter(AppState::GameView), setup);
        app.add_systems(app::Update, maintain_camera_system.run_if(in_state(AppState::GameView)));
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        super::Owned,
        Camera3dBundle {
            transform: Transform::from_xyz(0., 0., -5.).looking_at(Vec3::ZERO, Vec3::Y),
            ..<_>::default()
        },
    ));
}

fn maintain_camera_system(
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    camera_query.single_mut().rotate_y(time.delta_seconds());
}
