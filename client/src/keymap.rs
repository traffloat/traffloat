use enum_map::{Enum, EnumMap};
use shrev::EventChannel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enum)]
pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    RotLeft,
    RotRight,
    RotUp,
    RotDown,
    MoveFront,
    MoveBack,
    ZoomIn,
    ZoomOut,
    RotAntiClock,
    RotClock,
    Shift,
}

impl Action {
    pub fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "KeyW" => Self::MoveUp,
            "KeyS" => Self::MoveDown,
            "KeyA" => Self::MoveLeft,
            "KeyD" => Self::MoveRight,
            "KeyX" => Self::MoveFront,
            "KeyZ" => Self::MoveBack,
            "KeyQ" => Self::RotAntiClock,
            "KeyE" => Self::RotClock,
            "ShiftLeft" => Self::Shift,
            "ShiftRight" => Self::Shift,
            _ => return None,
        })
    }

    pub fn shift(self) -> Self {
        match self {
            Self::MoveUp => Self::RotUp,
            Self::MoveDown => Self::RotDown,
            Self::MoveLeft => Self::RotLeft,
            Self::MoveRight => Self::RotRight,
            Self::MoveFront => Self::ZoomIn,
            Self::MoveBack => Self::ZoomOut,
            _ => self,
        }
    }
}

pub struct ActionEvent {
    pub action: Action,
    pub active: bool,
}

#[derive(Debug, Default)]
pub struct CurrentActions(pub EnumMap<Action, bool>);

impl CurrentActions {
    pub fn actions(&self) -> impl Iterator<Item = Action> + '_ {
        let shift = self.0[Action::Shift];
        self.0
            .iter()
            .filter_map(|(action, value)| match value {
                true => Some(action),
                false => None,
            })
            .map(move |action| match shift {
                true => action.shift(),
                false => action,
            })
            .filter(|&action| action != Action::Shift)
    }
}

pub struct KeymapSystem {
    action_reader: shrev::ReaderId<ActionEvent>,
}

impl KeymapSystem {
    pub fn new(world: &mut specs::World) -> Self {
        use specs::SystemData;

        <Self as specs::System<'_>>::SystemData::setup(world);
        Self {
            action_reader: world
                .get_mut::<EventChannel<ActionEvent>>()
                .expect("Action channel initialized in setup")
                .register_reader(),
        }
    }
}

impl<'a> specs::System<'a> for KeymapSystem {
    type SystemData = (
        specs::Read<'a, EventChannel<ActionEvent>>,
        specs::Write<'a, CurrentActions>,
    );

    fn run(&mut self, (action_channel, mut action_set): Self::SystemData) {
        for event in action_channel.read(&mut self.action_reader) {
            action_set.0[event.action] = event.active;
        }
    }
}

pub fn setup_specs((mut world, mut dispatcher): common::Setup) -> common::Setup {
    dispatcher = dispatcher.with(KeymapSystem::new(&mut world), "keymap", &[]);

    (world, dispatcher)
}
