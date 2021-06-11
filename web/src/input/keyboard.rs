//! Keyboard input handler

use enum_map::EnumMap;

/// The set of currently active actions
#[derive(Clone, Default, derive_more::Index, derive_more::IndexMut)]
pub struct ActionSet(pub EnumMap<Action, bool>);

/// The ActionSet in the last tick
#[derive(Default)]
pub struct PrevActionSet(pub ActionSet);

#[derive(Debug)]
pub struct KeyEvent {
    code: Action,
    down: bool,
}

#[derive(Debug, Clone, Copy, enum_map::Enum)]
pub enum Action {
    Left,
    Right,
    Up,
    Down,
    Forward,
    Backward,
    ZoomIn,
    ZoomOut,
    LeftClick,
    MiddleClick,
    RightClick,
}

impl Action {
    fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "KeyA" => Self::Left,
            "KeyD" => Self::Right,
            "KeyW" => Self::Up,
            "KeyS" => Self::Down,
            "Equal" => Self::ZoomIn,
            "Minus" => Self::ZoomOut,
            _ => return None,
        })
    }

    fn from_mouse(button: i16) -> Option<Self> {
        Some(match button {
            0 => Self::LeftClick,
            1 => Self::MiddleClick,
            2 => Self::RightClick,
            _ => return None,
        })
    }
}

impl KeyEvent {
    pub fn new(code: &str, down: bool) -> Option<Self> {
        Some(Self {
            code: Action::from_code(code)?,
            down,
        })
    }

    pub fn new_mouse(button: i16, down: bool) -> Option<Self> {
        Some(Self {
            code: Action::from_mouse(button)?,
            down,
        })
    }
}

#[codegen::system]
fn input(
    #[subscriber] key_events: impl Iterator<Item = KeyEvent>,
    #[resource] key_set: &mut ActionSet,
    #[resource] prev_key_set: &mut PrevActionSet,
) {
    prev_key_set.0 = key_set.clone();

    #[allow(clippy::indexing_slicing)]
    for event in key_events {
        key_set[event.code] = event.down;
    }
}

pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(input_setup)
}
