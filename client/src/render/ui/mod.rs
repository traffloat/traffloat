//! Renders user interface.

use derive_new::new;
use legion::world::SubWorld;
use legion::EntityStore;

use super::Dimension;
use crate::input;
use traffloat::graph;

pub mod node;
mod wrapper;
pub use wrapper::*;

/// Stores setup data for the ui layer.
#[derive(new)]
pub struct Canvas {
    context: web_sys::CanvasRenderingContext2d,
}

impl Canvas {
    /// Resets the canvas.
    pub fn reset(&self, dim: &Dimension) {
        self.context
            .clear_rect(0., 0., dim.width.into(), dim.height.into());
    }
}

#[codegen::system]
#[read_component(graph::NodeName)]
#[thread_local]
fn draw(
    #[resource] cursor_target: &input::mouse::Target,
    world: &mut SubWorld,
    #[resource] updater_ref: &UpdaterRef,
) {
    let info = if let Some(entity) = cursor_target.entity() {
        let node_name = world
            .entry_ref(entity)
            .expect("Target entity does not exist") // TODO what if user is hovering over node while deleting it?
            .into_component::<graph::NodeName>()
            .expect("Component NodeName does not exist in target entity");
        Some(node::Props {
            node_name: node_name.name().to_string(),
        })
    } else {
        None
    };

    updater_ref.call(Update::SetNodeInfo(info));
}

/// Sets up legion ECS for debug info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup)
}
