#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumCount)]
pub enum KeyAction {
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
    Shift,
}

impl KeyAction {
    pub fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "KeyW" => Self::MoveUp,
            "KeyS" => Self::MoveDown,
            "KeyA" => Self::MoveLeft,
            "KeyD" => Self::MoveRight,
            "KeyX" => Self::MoveFront,
            "KeyZ" => Self::MoveBack,
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
