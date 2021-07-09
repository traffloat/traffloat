//! Handles mouse interaction.

use legion::Entity;

use super::ScreenPosition;
use crate::camera::Camera;
use crate::render;
use traffloat::shape::Shape;
use traffloat::space::Position;

/// Resource storing the position of the mouse, in the range [0, 1]^2.
#[derive(Debug, getset::CopyGetters)]
pub struct CursorPosition {
    /// Position of the mouse.
    #[getset(get_copy = "pub")]
    position: ScreenPosition,
}

impl CursorPosition {
    /// Creates a new cursor position.
    ///
    /// Parameters are identical to those of [`ScreenPosition::new`].
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            position: ScreenPosition::new(x, y),
        }
    }
}

impl Default for CursorPosition {
    fn default() -> Self {
        Self::new(0.5, 0.5)
    }
}

/// Resource storing the line segment below the cursor.
#[derive(getset::CopyGetters)]
pub struct Segment {
    /// The point closest to the camera.
    #[getset(get_copy = "pub")]
    proximal: Position,
    /// The point furthest from but still visible to the camera.
    #[getset(get_copy = "pub")]
    distal: Position,
}

impl Default for Segment {
    fn default() -> Self {
        Self {
            proximal: Position::new(0., 0., 0.),
            distal: Position::new(0., 0., -1.),
        }
    }
}

#[codegen::system]
fn trace_segment(
    #[resource] mode: &super::Mode,
    #[resource] cursor: &CursorPosition,
    #[resource] camera: &Camera,
    #[resource] segment: &mut Segment,

    #[debug("Mouse", "Proximal")] proximal_debug: &codegen::DebugEntry,
    #[debug("Mouse", "Distal")] distal_debug: &codegen::DebugEntry,
) {
    if !mode.needs_cursor_segment() {
        return;
    }

    let (proximal, distal) = camera.project_mouse(cursor.position().x(), cursor.position().y());
    segment.proximal = proximal;
    segment.distal = distal;

    codegen::update_debug!(
        proximal_debug,
        "({:.1}, {:.1}, {:.1})",
        segment.proximal().x(),
        segment.proximal().y(),
        segment.proximal().z(),
    );
    codegen::update_debug!(
        distal_debug,
        "({:.1}, {:.1}, {:.1})",
        segment.distal().x(),
        segment.distal().y(),
        segment.distal().z(),
    );
}

/// Resource storing the entity targeted by the cursor.
#[derive(Debug, Default, getset::CopyGetters, getset::Setters)]
pub struct Target {
    /// The target entity pointed by the cursor,
    /// or `None` if none can be detected.
    ///
    /// The value consists of an `f64` indicating the depth of the object
    /// relative to the camera focus and rendering distance,
    /// as well as an `Entity` indicating the entity ID of the object.
    #[getset(get_copy = "pub")]
    #[getset(set = "pub")]
    target: Option<(f64, Entity)>,
}

impl Target {
    /// Whether the passed entity is targeted
    pub fn is_entity(&self, entity: &Entity) -> bool {
        matches!(self.target(), Some((_, target)) if target == *entity)
    }
}

#[codegen::system]
#[read_component(Position)]
#[read_component(Shape)]
fn trace_entity(
    world: &legion::world::SubWorld,
    #[resource] mode: &super::Mode,
    #[resource] segment: &Segment,
    #[resource] cursor_type: &mut render::CursorType,
    #[resource] cursor_target: &mut Target,

    #[debug("Mouse", "Target entity")] target_debug: &codegen::DebugEntry,
    #[debug("Mouse", "Target depth")] target_depth_debug: &codegen::DebugEntry,
) {
    use legion::IntoQuery;

    if !mode.needs_cursor_entity() {
        return;
    }

    let mut last_depth = 2.0;
    cursor_target.set_target(None);
    for (&entity, &position, shape) in <(Entity, &Position, &Shape)>::query().iter(world) {
        let transform = shape.inv_transform(position);
        let proximal = transform.transform_point(&segment.proximal.0);
        let distal = transform.transform_point(&segment.distal.0);
        if let Some(depth) = shape.unit().between(proximal, distal) {
            if depth < last_depth {
                last_depth = depth;
                cursor_target.set_target(Some((depth, entity)));
            }
        }
    }

    if let Some((depth, entity)) = cursor_target.target() {
        codegen::update_debug!(target_debug, "{:?}", entity);
        codegen::update_debug!(target_depth_debug, "{:.1}", depth);
    } else {
        codegen::update_debug!(target_debug, "None");
        codegen::update_debug!(target_depth_debug, "None");
    }

    cursor_type.set_name(if cursor_target.target().is_some() {
        "pointer"
    } else {
        "initial"
    });
}

/// Sets up legion ECS for mouse input handling.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(trace_segment_setup).uses(trace_entity_setup)
}
