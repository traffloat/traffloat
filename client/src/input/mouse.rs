//! Keyboard input handler

use enum_map::EnumMap;

use crate::camera::Camera;
use traffloat::types::{Position, Vector};

#[derive(Debug)]
pub enum MouseEvent {
    Move { x: f64, y: f64, dx: f64, dy: f64 },
}

/// The position pointed by the cursor
#[derive(Default)]
pub struct CursorPosition {
    /// The position value, or None if MouseEvent was never fired
    pub value: Option<Position>,
}

#[legion::system]
#[allow(clippy::indexing_slicing)]
fn input(
    #[state] reader: &mut shrev::ReaderId<MouseEvent>,
    #[state] current_cursor: &mut Option<(f64, f64)>,
    #[resource] camera: &Camera,
    #[resource] chan: &mut shrev::EventChannel<MouseEvent>,
    #[resource] cursor: &mut CursorPosition,
) {
    for event in chan.read(reader) {
        match event {
            MouseEvent::Move { x, y, .. } => {
                *current_cursor = Some((*x, *y));
            }
        }
    }

    if let Some((x, y)) = *current_cursor {
        let canvas_pos = Vector::new(x, y);
        let real_pos = camera.image_unit_to_real(canvas_pos);
        cursor.value = Some(real_pos);
    }
}

pub fn setup_ecs(mut setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    let reader = setup.subscribe::<MouseEvent>();
    setup
        .resource(CursorPosition::default())
        .system(input_system(reader, None))
}
