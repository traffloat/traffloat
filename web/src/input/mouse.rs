//! Keyboard input handler

use std::ops::Sub;

use enum_map::EnumMap;

use super::keyboard;

/// Mouse motion event
///
/// Mouse clicking events are dispatched as key events instead.
#[derive(Debug)]
pub enum MouseEvent {
    Move { x: f64, y: f64 },
}

/// Mouse scrolling event
#[derive(Debug)]
pub struct WheelEvent {
    pub delta: f64,
}

/// The screen position of the cursor, scaled to the [0, 1]^2 square.
///
/// (0, 0) is the lower left corner of the screen.
#[derive(Debug, Clone, Copy, Default)]
pub struct CursorPosition(pub Option<ScreenPosition>);

#[derive(Debug, Clone, Copy, Default)]
pub struct ScreenPosition {
    pub x: f64,
    pub y: f64,
}

impl Sub<ScreenPosition> for ScreenPosition {
    type Output = (f64, f64);

    fn sub(self, other: Self) -> Self::Output {
        (self.x - other.x, self.y - other.y)
    }
}

/// Marker component for clickable entities
pub struct Clickable(pub bool);

#[derive(Debug, Clone, Copy)]
enum DragState {
    None,
    Drag {
        start: ScreenPosition,
        last: ScreenPosition,
    },
}

impl Default for DragState {
    fn default() -> Self {
        DragState::None
    }
}

pub enum DragEvent {
    Start {
        action: keyboard::Action,
        position: ScreenPosition,
    },
    Move {
        action: keyboard::Action,
        start: ScreenPosition,
        prev: ScreenPosition,
        now: ScreenPosition,
    },
    End {
        action: keyboard::Action,
        start: ScreenPosition,
        end: ScreenPosition,
    },
}

#[codegen::system]
#[allow(clippy::too_many_arguments)]
#[thread_local]
fn input(
    #[resource] cursor: &ScreenPosition,
    #[resource] actions: &keyboard::ActionSet,
    #[resource] drag_events: &mut shrev::EventChannel<DragEvent>,
    #[state(Default::default())] drag_states: &mut EnumMap<keyboard::Action, DragState>,
) {
    #[allow(clippy::indexing_slicing)]
    for &action in &[
        keyboard::Action::LeftClick,
        keyboard::Action::MiddleClick,
        keyboard::Action::RightClick,
    ] {
        let state = &mut drag_states[action];

        match (*state, actions[action]) {
            (DragState::None, false) => {} // never dragging
            (DragState::None, true) => {
                // start dragging
                *state = DragState::Drag {
                    start: *cursor,
                    last: *cursor,
                };
                drag_events.single_write(DragEvent::Start {
                    action,
                    position: *cursor,
                });
            }
            (DragState::Drag { start, last }, true) => {
                // continue dragging
                *state = DragState::Drag {
                    start,
                    last: *cursor,
                };
                drag_events.single_write(DragEvent::Move {
                    action,
                    start,
                    prev: last,
                    now: *cursor,
                });
            }
            (DragState::Drag { start, last }, false) => {
                // stop dragging
                *state = DragState::None;
                drag_events.single_write(DragEvent::Move {
                    action,
                    start,
                    prev: last,
                    now: *cursor,
                });
                drag_events.single_write(DragEvent::End {
                    action,
                    start,
                    end: *cursor,
                });
            }
        }
    }
}

pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.publish::<DragEvent>().uses(input_setup)
}
