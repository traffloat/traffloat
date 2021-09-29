//! Handles mouse interaction.

use legion::Entity;

use super::{keyboard, ScreenPosition};
use crate::camera::Camera;
use crate::render;
use traffloat::space::Position;
use traffloat::{appearance, edge, node};

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
        Self { position: ScreenPosition::new(x, y) }
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
        Self { proximal: Position::new(0., 0., 0.), distal: Position::new(0., 0., -1.) }
    }
}

#[codegen::system(Input)]
fn trace_segment(
    #[resource] cursor: &CursorPosition,
    #[resource] camera: &Camera,
    #[resource] segment: &mut Segment,

    #[debug("Mouse", "Proximal")] proximal_debug: &codegen::DebugEntry,
    #[debug("Mouse", "Distal")] distal_debug: &codegen::DebugEntry,
) {
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

/// Resource storing the entity that the cursor hovers over.
#[derive(Debug, Clone, Default, getset::CopyGetters, getset::Setters)]
pub struct HoverTarget {
    /// The target entity pointed by the cursor,
    /// or `None` if none can be detected.
    #[getset(get_copy = "pub")]
    #[getset(set = "pub")]
    entity: Option<Entity>,
}

#[codegen::system(Response)]
#[read_component(Position)]
#[read_component(appearance::Appearance)]
#[read_component(node::Id)]
#[read_component(edge::Id)]
#[read_component(edge::Size)]
fn trace_entity(
    world: &legion::world::SubWorld,
    #[resource] segment: &Segment,
    #[resource] cursor_type: &mut render::CursorType,
    #[resource] hover_target: &mut HoverTarget,
    #[resource] focus_target: &mut super::FocusTarget,
    #[subscriber] click_sub: impl Iterator<Item = keyboard::SingleClick>,

    #[debug("Mouse", "Target")] target_debug: &codegen::DebugEntry,
) {
    use legion::IntoQuery;

    hover_target.set_entity(None);
    let mut last_depth = 2.0;
    for (&entity, &position, appearance) in
        <(Entity, &Position, &appearance::Appearance)>::query().iter(world)
    {
        for component in appearance.components() {
            let transform = component.inv_transform(position);
            let proximal = transform.transform_point(&segment.proximal.0);
            let distal = transform.transform_point(&segment.distal.0);

            if let Some(depth) = component.unit().between(proximal, distal) {
                if depth < last_depth {
                    last_depth = depth;
                    hover_target.set_entity(Some(entity));
                }
            }
        }
    }

    for (&entity, edge, size) in <(Entity, &edge::Id, &edge::Size)>::query().iter(world) {
        let unit = edge::tf(edge, size, &*world, false);
        let proximal = unit.transform_point(&segment.proximal.0);
        let distal = unit.transform_point(&segment.distal.0);

        if let Some(depth) = appearance::Unit::Cylinder.between(proximal, distal) {
            if depth < last_depth {
                last_depth = depth;
                hover_target.set_entity(Some(entity));
            }
        }
    }

    if let Some(entity) = hover_target.entity() {
        codegen::update_debug!(target_debug, "{:?}", entity);
    } else {
        codegen::update_debug!(target_debug, "None");
    }

    cursor_type.set_name(if hover_target.entity().is_some() { "pointer" } else { "initial" });

    let has_click =
        click_sub.filter(|click| click.command() == keyboard::Command::LeftClick).count() > 0; // consume the whole iterator without short-circuiting
    if has_click {
        focus_target.set_entity(hover_target.entity());
    }
}

/// Sets up legion ECS for mouse input handling.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(trace_segment_setup).uses(trace_entity_setup)
}
