//! Keyboard input handler

use enum_map::EnumMap;
use legion::Entity;

use crate::camera::Camera;
use crate::render;
use traffloat::shape::Shape;
use traffloat::types::{Position, Vector};

#[derive(Debug)]
pub enum MouseEvent {
    Move { x: f64, y: f64, dx: f64, dy: f64 },
}

/// The object pointed by the cursor
#[derive(Default)]
pub struct CursorPosition {
    /// The pointed position, or None if MouseEvent was never fired
    pub pos: Option<Position>,
    /// The pointed entity, or None if mouse is not pointing to a Clickable
    pub entity: Option<Entity>,
}

/// Marker component for clickable entities
pub struct Clickable;

#[legion::system]
#[allow(clippy::indexing_slicing)]
#[read_component(Shape)]
#[read_component(Position)]
#[read_component(Clickable)]
fn input(
    world: &mut legion::world::SubWorld,
    #[state] reader: &mut shrev::ReaderId<MouseEvent>,
    #[state] current_cursor: &mut Option<(f64, f64)>,
    #[resource] camera: &Camera,
    #[resource] chan: &shrev::EventChannel<MouseEvent>,
    #[resource] cursor: &mut CursorPosition,
    #[resource] dim: &render::Dimension,
    #[resource] comm: &render::Comm,
) {
    for event in chan.read(reader) {
        match event {
            MouseEvent::Move { x, y, .. } => {
                *current_cursor = Some((*x, *y));
            }
        }
    }

    if let Some((x, y)) = *current_cursor {
        use legion::IntoQuery;

        let canvas_pos = Vector::new(x, y);
        let real_pos = camera.image_unit_to_real(canvas_pos, dim.aspect());
        cursor.pos = Some(real_pos);

        cursor.entity = None;
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
                cursor.entity = Some(*entity);
                comm.canvas_cursor_type.set("pointer");
                break;
            }
        }
    }
}

pub fn setup_ecs(mut setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    let reader = setup.subscribe::<MouseEvent>();
    setup
        .resource(CursorPosition::default())
        .system_local(input_system(reader, None))
}
