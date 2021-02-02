//! Keyboard input handler

use legion::Entity;

use crate::camera::Camera;
use crate::render;
use traffloat::shape::Shape;
use traffloat::types::{Position, Vector};

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

/// The object pointed by the cursor
pub struct CursorPosition {
    /// The pointed position, or None if MouseEvent was never fired
    pub pos: Option<Position>,
    /// The pointed entity, or the canvas position if mouse is not pointing to a Clickable
    ///
    /// This value is invalid if `pos` is None.
    pub entity: Result<Entity, (f64, f64)>,
}

impl Default for CursorPosition {
    fn default() -> Self {
        Self {
            pos: None,
            entity: Err((0., 0.)),
        }
    }
}

/// Marker component for clickable entities
pub struct Clickable;

#[codegen::system]
#[allow(clippy::too_many_arguments)]
#[read_component(Shape)]
#[read_component(Position)]
#[read_component(Clickable)]
#[thread_local]
fn input(
    world: &mut legion::world::SubWorld,
    #[subscriber] mouse_events: impl Iterator<Item = MouseEvent>,
    #[state(None)] current_cursor: &mut Option<(f64, f64)>,
    #[resource] camera: &Camera,
    #[resource] cursor: &mut CursorPosition,
    #[resource] dim: &render::Dimension,
    #[resource] comm: &render::Comm,
) {
    for event in mouse_events {
        match event {
            MouseEvent::Move { x, y } => {
                *current_cursor = Some((*x, *y));
            }
        }
    }

    if let Some((x, y)) = *current_cursor {
        use legion::IntoQuery;

        let canvas_pos = Vector::new(x, y);
        let real_pos = camera.image_unit_to_real(canvas_pos, dim.aspect());
        cursor.pos = Some(real_pos);

        cursor.entity = Err((x, y));
        comm.canvas_cursor_type.set("initial");
        for (entity, &position, shape, _) in
            <(Entity, &Position, &Shape, &Clickable)>::query().iter(world)
        {
            let point = shape
                .transform(position)
                .try_inverse()
                .expect("Transformation matrix is singular")
                .transform_point(&real_pos.0);
            if shape.unit.contains(point) {
                cursor.entity = Ok(*entity);
                comm.canvas_cursor_type.set("pointer");
                break;
            }
        }
    }
}

pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.resource(CursorPosition::default()).uses(input_setup)
}
