//! Handles keyboard input.

#![allow(clippy::indexing_slicing)] // this module uses EnumMap extensively.

use derive_new::new;
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
    key: RawKey,
    /// Whether the key is pressed down or up.
    #[getset(get_copy = "pub")]
    down: bool,
}

impl RawKeyEvent {
    /// The code of the event.
    pub fn key(&self) -> RawKey<&str> {
        self.key.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A raw key used by the yew layer.
pub enum RawKey<S = String> {
    /// A keyboard key.
    Key(S),
    /// A mouse button.
    Mouse(i16),
}

impl RawKey<String> {
    /// Changes the key `String` to a `&str` if any.
    pub fn as_ref(&self) -> RawKey<&str> {
        match self {
            Self::Key(s) => RawKey::Key(s.as_str()),
            &Self::Mouse(button) => RawKey::Mouse(button),
        }
    }
}

/// Commands interpreted by the keymap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, enum_map::Enum)]
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
    /// The generic left click command
    LeftClick,
    /// The generic middle click command
    MiddleClick,
    /// The generic right click command
    RightClick,
}

impl Command {
    /// Converts a `KeyboardEvent.code` to a `Command` if possible.
    pub fn from_key(key: RawKey<&str>) -> Option<Command> {
        Some(match key {
            RawKey::Key("KeyW") => Command::MoveUp,
            RawKey::Key("KeyS") => Command::MoveDown,
            RawKey::Key("KeyA") => Command::MoveLeft,
            RawKey::Key("KeyD") => Command::MoveRight,
            RawKey::Key("KeyZ") => Command::MoveFront,
            RawKey::Key("KeyX") => Command::MoveBack,
            RawKey::Key("Equal") => Command::ZoomIn,
            RawKey::Key("Minus") => Command::ZoomOut,
            RawKey::Key("ShiftLeft") => Command::RotationMask,
            RawKey::Mouse(0) => Command::LeftClick,
            RawKey::Mouse(1) => Command::MiddleClick,
            RawKey::Mouse(2) => Command::RightClick,
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
    pub fn set(&mut self, down: bool, now: time::Instant) -> Option<ClickType> {
        self.active = down;
        if down {
            let prev = self.last_down;
            self.last_down = now;
            if now - prev < config::DOUBLE_CLICK_INTERVAL {
                Some(ClickType::Double)
            } else {
                Some(ClickType::Single)
            }
        } else {
            self.last_up = now;
            None
        }
    }
}

/// The type of clicking detected on command state updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickType {
    /// A single lcick
    Single,
    /// A double lcick
    Double,
}

/// A map storing the time since which a command was clicked or released.
pub type CommandStates = EnumMap<Command, CommandState>;

/// A single click event.
#[derive(new, getset::CopyGetters)]
pub struct SingleClick {
    /// The command clicked.
    #[getset(get_copy = "pub")]
    command: Command,
}

/// A double click event.
#[derive(new, getset::CopyGetters)]
pub struct DoubleClick {
    /// The command clicked.
    #[getset(get_copy = "pub")]
    command: Command,
}

#[codegen::system]
fn track_states(
    #[resource] states: &mut CommandStates,
    #[resource] clock: &time::Clock,
    #[subscriber] raw_key_events: impl Iterator<Item = RawKeyEvent>,
    #[resource] single_click_pub: &mut shrev::EventChannel<SingleClick>,
    #[resource] double_click_pub: &mut shrev::EventChannel<DoubleClick>,
) {
    let now = clock.now();
    for event in raw_key_events {
        if let Some(command) = Command::from_key(event.key()) {
            match states[command].set(event.down(), now) {
                Some(ClickType::Single) => {
                    single_click_pub.single_write(SingleClick::new(command));
                }
                Some(ClickType::Double) => {
                    double_click_pub.single_write(DoubleClick::new(command));
                }
                _ => {}
            }
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
            yaw -= config::WASD_ROTATION_VELOCITY * dt;
        }
        if commands[Command::MoveRight].active() {
            yaw += config::WASD_ROTATION_VELOCITY * dt;
        }
        if commands[Command::MoveDown].active() {
            pitch += config::WASD_ROTATION_VELOCITY * dt;
        }
        if commands[Command::MoveUp].active() {
            pitch -= config::WASD_ROTATION_VELOCITY * dt;
        }
        if commands[Command::MoveFront].active() {
            roll -= config::WASD_ROTATION_VELOCITY * dt;
        }
        if commands[Command::MoveBack].active() {
            roll += config::WASD_ROTATION_VELOCITY * dt;
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
                .transpose()
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
