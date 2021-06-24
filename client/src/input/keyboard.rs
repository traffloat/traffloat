//! Handles keyboard input.

use enum_map::{Enum, EnumMap};
use typed_builder::TypedBuilder;

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
pub fn track_states(
    #[resource] states: &mut CommandStates,
    #[resource] clock: &time::Clock,
    #[subscriber] raw_key_events: impl Iterator<Item = RawKeyEvent>,
) {
    let now = clock.now();
    for event in raw_key_events {
        if let Some(command) = Command::from_code(&event.code()) {
            states[command].set(event.down(), now);
        }
    }
}

/// Sets up legion ECS for keyboard input handling.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(track_states_setup)
}
