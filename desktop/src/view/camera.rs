use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use bevy::app::{self, App};
use bevy::color::Color;
use bevy::core_pipeline::core_3d::{Camera3d, Camera3dBundle};
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Query, Res};
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;
use bevy::math::{Quat, Vec3};
use bevy::pbr;
use bevy::pbr::light_consts::lux;
use bevy::state::condition::in_state;
use bevy::state::state;
use bevy::time::Time;
use bevy::transform::components::Transform;

use super::InputSystemSet;
use crate::AppState;

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(state::OnEnter(AppState::GameView), setup);
        app.add_systems(
            app::Update,
            input_move_camera_system.run_if(in_state(AppState::GameView)).in_set(InputSystemSet),
        );

        app.insert_resource(pbr::AmbientLight { color: Color::WHITE, brightness: 20. });
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        super::Owned,
        Camera3dBundle {
            transform: Transform::from_xyz(0., 0., -5.).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
    ));

    commands.spawn((
        super::Owned,
        pbr::DirectionalLightBundle {
            directional_light: pbr::DirectionalLight {
                color: Color::WHITE,
                illuminance: lux::CLEAR_SUNRISE,
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0., 1., -1.),
                rotation: Quat::from_rotation_x(-FRAC_PI_2),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

fn input_move_camera_system(
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut camera) = camera_query.get_single_mut() else { return };

    let move_speed = time.delta_seconds() * 5.;
    let rotate_speed = time.delta_seconds() * FRAC_PI_4;

    let is_rotate = keys.pressed(KeyCode::ShiftLeft);

    if keys.pressed(KeyCode::KeyW) {
        if is_rotate {
            camera.rotate_local_x(rotate_speed);
        } else {
            let delta = camera.up() * move_speed;
            camera.translation += delta;
        }
    }

    if keys.pressed(KeyCode::KeyS) {
        if is_rotate {
            camera.rotate_local_x(-rotate_speed);
        } else {
            let delta = camera.down() * move_speed;
            camera.translation += delta;
        }
    }

    if keys.pressed(KeyCode::KeyA) {
        if is_rotate {
            camera.rotate_local_y(rotate_speed);
        } else {
            let delta = camera.left() * move_speed;
            camera.translation += delta;
        }
    }

    if keys.pressed(KeyCode::KeyD) {
        if is_rotate {
            camera.rotate_local_y(-rotate_speed);
        } else {
            let delta = camera.right() * move_speed;
            camera.translation += delta;
        }
    }

    if keys.pressed(KeyCode::KeyZ) {
        if is_rotate {
            camera.rotate_local_z(-rotate_speed);
        } else {
            let delta = camera.forward() * move_speed;
            camera.translation += delta;
        }
    }

    if keys.pressed(KeyCode::KeyX) {
        if is_rotate {
            camera.rotate_local_z(rotate_speed);
        } else {
            let delta = camera.back() * move_speed;
            camera.translation += delta;
        }
    }
}
