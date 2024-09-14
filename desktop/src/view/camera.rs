use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use bevy::app::{self, App};
use bevy::color::Color;
use bevy::core_pipeline::core_3d::{Camera3d, Camera3dBundle};
use bevy::diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic};
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Query, Res};
use bevy::hierarchy::BuildChildren;
use bevy::input::keyboard::KeyCode;
use bevy::input::ButtonInput;
use bevy::math::{Quat, Vec3};
use bevy::pbr;
use bevy::pbr::light_consts::lux;
use bevy::render::camera;
use bevy::state::condition::in_state;
use bevy::state::state;
use bevy::time::Time;
use bevy::transform::components::Transform;

use super::{diagnostics, InputSystemSet};
use crate::AppState;

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(state::OnEnter(AppState::GameView), setup);
        app.add_systems(
            app::Update,
            input_move_camera_system.run_if(in_state(AppState::GameView)).in_set(InputSystemSet),
        );

        app.add_systems(app::Startup, register_camera_diagnostic_system);
        app.add_systems(app::Update, update_camera_diagnostic_system);
        app.register_diagnostic(Diagnostic::new(DIAG_PATH_POS_X));
        app.register_diagnostic(Diagnostic::new(DIAG_PATH_POS_Y));
        app.register_diagnostic(Diagnostic::new(DIAG_PATH_POS_Z));
        app.register_diagnostic(Diagnostic::new(DIAG_PATH_FACE_X));
        app.register_diagnostic(Diagnostic::new(DIAG_PATH_FACE_Y));
        app.register_diagnostic(Diagnostic::new(DIAG_PATH_FACE_Z));
        app.register_diagnostic(Diagnostic::new(DIAG_PATH_ZOOM));

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
    mut camera_query: Query<(&mut Transform, &mut camera::Projection), With<Camera3d>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok((mut tf, mut proj)) = camera_query.get_single_mut() else { return };

    let move_speed = time.delta_seconds() * 5.;
    let rotate_speed = time.delta_seconds() * FRAC_PI_4;
    let zoom_speed = 1.7_f32.powf(time.delta_seconds());

    let is_rotate = keys.pressed(KeyCode::ShiftLeft);

    if keys.pressed(KeyCode::KeyW) {
        if is_rotate {
            tf.rotate_local_x(rotate_speed);
        } else {
            let delta = tf.up() * move_speed;
            tf.translation += delta;
        }
    }

    if keys.pressed(KeyCode::KeyS) {
        if is_rotate {
            tf.rotate_local_x(-rotate_speed);
        } else {
            let delta = tf.down() * move_speed;
            tf.translation += delta;
        }
    }

    if keys.pressed(KeyCode::KeyA) {
        if is_rotate {
            tf.rotate_local_y(rotate_speed);
        } else {
            let delta = tf.left() * move_speed;
            tf.translation += delta;
        }
    }

    if keys.pressed(KeyCode::KeyD) {
        if is_rotate {
            tf.rotate_local_y(-rotate_speed);
        } else {
            let delta = tf.right() * move_speed;
            tf.translation += delta;
        }
    }

    if keys.pressed(KeyCode::KeyZ) {
        if is_rotate {
            tf.rotate_local_z(-rotate_speed);
        } else {
            let delta = tf.forward() * move_speed;
            tf.translation += delta;
        }
    }

    if keys.pressed(KeyCode::KeyX) {
        if is_rotate {
            tf.rotate_local_z(rotate_speed);
        } else {
            let delta = tf.back() * move_speed;
            tf.translation += delta;
        }
    }

    if let camera::Projection::Perspective(ref mut proj) = *proj {
        if keys.pressed(KeyCode::Equal) {
            proj.fov /= zoom_speed;
        }

        if keys.pressed(KeyCode::Minus) {
            proj.fov = (proj.fov * zoom_speed).min(PI);
        }
    }
}

const DIAG_PATH_POS_X: DiagnosticPath = DiagnosticPath::const_new("traffloat/camera/source/x");
const DIAG_PATH_POS_Y: DiagnosticPath = DiagnosticPath::const_new("traffloat/camera/source/y");
const DIAG_PATH_POS_Z: DiagnosticPath = DiagnosticPath::const_new("traffloat/camera/source/z");

const DIAG_PATH_FACE_X: DiagnosticPath = DiagnosticPath::const_new("traffloat/camera/face/x");
const DIAG_PATH_FACE_Y: DiagnosticPath = DiagnosticPath::const_new("traffloat/camera/face/y");
const DIAG_PATH_FACE_Z: DiagnosticPath = DiagnosticPath::const_new("traffloat/camera/face/z");

const DIAG_PATH_ZOOM: DiagnosticPath = DiagnosticPath::const_new("traffloat/camera/zoom");

fn register_camera_diagnostic_system(mut commands: Commands) {
    commands
        .spawn(
            diagnostics::DisplayGroup::builder()
                .vertical_priority(20)
                .id("camera")
                .label("Camera")
                .build(),
        )
        .with_children(|b| {
            b.spawn(
                diagnostics::Display::builder()
                    .horizontal_priority(0)
                    .label("Pos")
                    .target(DIAG_PATH_POS_X)
                    .target(DIAG_PATH_POS_Y)
                    .target(DIAG_PATH_POS_Z)
                    .build(),
            );
            b.spawn(
                diagnostics::Display::builder()
                    .horizontal_priority(1)
                    .label("Facing")
                    .target(DIAG_PATH_FACE_X)
                    .target(DIAG_PATH_FACE_Y)
                    .target(DIAG_PATH_FACE_Z)
                    .build(),
            );
            b.spawn(
                diagnostics::Display::builder()
                    .horizontal_priority(2)
                    .label("Zoom")
                    .target(DIAG_PATH_ZOOM)
                    .build(),
            );
        });
}

fn update_camera_diagnostic_system(
    mut diagnostics: Diagnostics,
    camera_query: Query<(&Transform, &camera::Projection), With<Camera3d>>,
) {
    let Ok((tf, proj)) = camera_query.get_single() else { return };

    diagnostics.add_measurement(&DIAG_PATH_POS_X, || tf.translation.x.into());
    diagnostics.add_measurement(&DIAG_PATH_POS_Y, || tf.translation.y.into());
    diagnostics.add_measurement(&DIAG_PATH_POS_Z, || tf.translation.z.into());

    let facing = tf.rotation.mul_vec3(-Vec3::Z);
    diagnostics.add_measurement(&DIAG_PATH_FACE_X, || facing.x.into());
    diagnostics.add_measurement(&DIAG_PATH_FACE_Y, || facing.y.into());
    diagnostics.add_measurement(&DIAG_PATH_FACE_Z, || facing.z.into());

    if let camera::Projection::Perspective(proj) = proj {
        diagnostics.add_measurement(&DIAG_PATH_ZOOM, || (PI / proj.fov).log2().into());
    }
}
