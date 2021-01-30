//! Keyboard input handler

use enum_map::EnumMap;

pub type ActionSet = EnumMap<Action, bool>;

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
    ZoomIn,
    ZoomOut,
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
}

impl KeyEvent {
    pub fn new(code: &str, down: bool) -> Option<Self> {
        Some(Self {
            code: Action::from_code(code)?,
            down,
        })
    }
}

#[legion::system]
#[allow(clippy::indexing_slicing)]
fn input(
    #[state] reader: &mut shrev::ReaderId<KeyEvent>,
    #[state] key_set: &mut ActionSet,
    #[resource] chan: &mut shrev::EventChannel<KeyEvent>,
) {
    for event in chan.read(reader) {
        key_set[event.code] = event.down;
    }
}

pub fn setup_ecs(mut setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    let reader = setup.subscribe::<KeyEvent>();
    setup
        .resource(ActionSet::default())
        .system(input_system(reader, EnumMap::default()))
}
