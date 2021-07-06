//! Handles keyboard input.

use enum_map::EnumMap;
use typed_builder::TypedBuilder;

use crate::camera::Camera;
use crate::config;
use crate::render;
use traffloat::space::Vector;
use traffloat::time;

/// A raw key event from the yew layer.
#[derive(TypedBuilder, getset::Getters, getset::CopyGetters)]
pub struct RawKeyEvent {
    /// The code of the event.
    #[getset(get = "pub")]
    code: String,
    /// Whether the key is pressed down or up.
    #[getset(get_copy = "pub")]
    down: bool,
}

/// Commands interpreted by the keymap.
#[derive(Debug, Clone, Copy, enum_map::Enum)]
pub enum Command {
    /// Move leftwards
    ///
    /// With rotation mask, rotate leftwards (rotation axis upwards)
    MoveLeft,
    /// Move rightwards
    ///
    /// With rotation mask, rotate rightwards (rotation axis downwards)
    MoveRight,
    /// Move upwards
    ///
    /// With rotation mask, rotate upwards (rotation axis rightwards)
    MoveUp,
    /// Move downwards
    ///
    /// With rotation mask, rotate downwards (rotation axis leftwards)
    MoveDown,
    /// Move forward
    ///
    /// With rotation mask, rotate clockwise (rotation axis forward)
    MoveFront,
    /// Move backward
    ///
    /// With rotation mask, rotate counter-clockwise (rotation axis backward)
    MoveBack,
    /// Rotate leftwards
    RotationMask,
    /// Zoom in
    ZoomIn,
    /// Zoom out
    ZoomOut,
}

impl Command {
    /// Converts a `KeyboardEvent.code` to a `Command` if possible.
    pub fn from_code(code: &str) -> Option<Command> {
        Some(match code {
            "KeyW" => Command::MoveUp,
            "KeyS" => Command::MoveDown,
            "KeyA" => Command::MoveLeft,
            "KeyD" => Command::MoveRight,
            "KeyZ" => Command::MoveFront,
            "KeyX" => Command::MoveBack,
            "Equal" => Command::ZoomIn,
            "Minus" => Command::ZoomOut,
            "ShiftLeft" => Command::RotationMask,
            _ => return None,
        })
    }
}

/// The state of a command.
#[derive(Default, getset::CopyGetters)]
pub struct CommandState {
    /// Whether the command is currently active.
    #[getset(get_copy = "pub")]
    active: bool,
    /// The last instant at which the command state changed from inactive to active.
    #[getset(get_copy = "pub")]
    last_down: time::Instant,
    /// The last instant at which the command state changed from active to inactive.
    #[getset(get_copy = "pub")]
    last_up: time::Instant,
}

impl CommandState {
    /// Sets the command state and updates the last activation/deactivation time.
    pub fn set(&mut self, down: bool, now: time::Instant) {
        self.active = down;
        if down {
            self.last_down = now;
        } else {
            self.last_up = now;
        }
    }
}

/// A map storing the time since which a command was clicked or released.
pub type CommandStates = EnumMap<Command, CommandState>;

#[codegen::system]
fn track_states(
    #[resource] states: &mut CommandStates,
    #[resource] clock: &time::Clock,
    #[subscriber] raw_key_events: impl Iterator<Item = RawKeyEvent>,
) {
    let now = clock.now();
    for event in raw_key_events {
        if let Some(command) = Command::from_code(event.code()) {
            use std::ops::IndexMut;
            states.index_mut(command).set(event.down(), now);
        }
    }
}

#[codegen::system]
#[allow(clippy::indexing_slicing, clippy::too_many_arguments)]
fn move_camera(
    #[resource] camera: &mut Camera,
    #[resource] clock: &time::Clock,
    #[resource] commands: &CommandStates,
    #[resource(no_init)] dim: &render::Dimension,
) {
    let dt = clock.delta().value() as f64;

    if commands[Command::RotationMask].active() {
        let mut roll = 0.;
        let mut pitch = 0.;
        let mut yaw = 0.;
        if commands[Command::MoveLeft].active() {
            yaw -= config::WASD_ROTATION_VELOCITY;
        }
        if commands[Command::MoveRight].active() {
            yaw += config::WASD_ROTATION_VELOCITY;
        }
        if commands[Command::MoveDown].active() {
            pitch += config::WASD_ROTATION_VELOCITY;
        }
        if commands[Command::MoveUp].active() {
            pitch -= config::WASD_ROTATION_VELOCITY;
        }
        if commands[Command::MoveFront].active() {
            roll -= config::WASD_ROTATION_VELOCITY;
        }
        if commands[Command::MoveBack].active() {
            roll += config::WASD_ROTATION_VELOCITY;
        }
        if roll != 0. || pitch != 0. || yaw != 0. {
            let mat = nalgebra::Rotation3::from_euler_angles(pitch, yaw, roll).to_homogeneous();
            camera.set_rotation(mat * camera.rotation());
        }
    } else {
        let mut move_direction = Vector::new(0., 0., 0.);
        if commands[Command::MoveLeft].active() {
            move_direction += Vector::new(-config::WASD_LINEAR_VELOCITY * dt, 0., 0.);
        }
        if commands[Command::MoveRight].active() {
            move_direction += Vector::new(config::WASD_LINEAR_VELOCITY * dt, 0., 0.);
        }
        if commands[Command::MoveUp].active() {
            move_direction += Vector::new(0., config::WASD_LINEAR_VELOCITY * dt, 0.);
        }
        if commands[Command::MoveDown].active() {
            move_direction += Vector::new(0., -config::WASD_LINEAR_VELOCITY * dt, 0.);
        }
        if commands[Command::MoveFront].active() {
            move_direction += Vector::new(0., 0., -config::WASD_LINEAR_VELOCITY * dt);
        }
        if commands[Command::MoveBack].active() {
            move_direction += Vector::new(0., 0., config::WASD_LINEAR_VELOCITY * dt);
        }
        if move_direction != Vector::new(0., 0., 0.) {
            let dp = camera
                .rotation()
                .try_inverse()
                .expect("Rotation matrix is singular")
                .transform_vector(&move_direction);
            camera.set_focus(camera.focus() + dp);
        }
    }
    if commands[Command::ZoomIn].active() {
        camera.set_zoom(camera.zoom() + config::ZOOM_VELOCITY * dt);
    }
    if commands[Command::ZoomOut].active() {
        // camera.set_zoom(f64::max(camera.zoom() - config::ZOOM_VELOCITY * dt, 0.0001));
        camera.set_zoom(camera.zoom() - config::ZOOM_VELOCITY * dt);
    }

    /*
    for wheel in wheel_events {
        if wheel.delta > 0. {
            camera.set_zoom(camera.zoom() + config::SCROLL_VELOCITY);
        } else {
            camera.set_zoom(camera.zoom() - config::SCROLL_VELOCITY);
        }
    }
    */

    #[allow(clippy::float_cmp)] // we simply want to see if it *might* have changed
    if camera.aspect() != dim.aspect() {
        camera.set_aspect(dim.aspect());
    }

    /*
    for event in drag_events {
        let (action, prev, now) = match event {
            mouse::DragEvent::Move {
                action, prev, now, ..
            } => (action, prev, now),
            _ => continue,
        };

        let delta = *now - *prev;
        match action {
            Action::LeftClick => {
                // TODO rotation
            }
            Action::RightClick => {
                // TODO motion
            }
            _ => {} // unused
        }
    }
    */
}

/// Sets up legion ECS for keyboard input handling.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(track_states_setup).uses(move_camera_setup)
}
