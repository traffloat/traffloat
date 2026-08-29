use bevy::app::{self, App, Plugin};
use bevy::camera::{ImageRenderTarget, NormalizedRenderTarget};
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::observer;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, EntityCommands, Local, Query, Res};
use bevy::ecs::world::World;
use bevy::math::Vec2;
use bevy::picking::input::PointerInputSettings;
use bevy::picking::mesh_picking::{MeshPickingCamera, MeshPickingSettings};
use bevy::picking::pointer::{PointerAction, PointerId, PointerInput};
use bevy::picking::{PickingSettings, events as pick_event, pointer};

use crate::gui;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.insert_resource(PickingSettings { is_input_enabled: true, ..Default::default() });
        app.insert_resource(PointerInputSettings { is_mouse_enabled: false, ..Default::default() });
        app.insert_resource(MeshPickingSettings { require_markers: true, ..Default::default() });
    }
}

pub fn add_observers(entity: &mut EntityCommands) {
    let id = entity.id();
    entity.observe(
        move |event: observer::On<pick_event::Pointer<pick_event::Click>>,
              mut focus_writer: MessageWriter<gui::RequestCameraFocus>,
              mut open_writer: MessageWriter<gui::OpenViewableInfo>,
              camera: Res<gui::CameraInteraction>| {
            open_writer.write(gui::OpenViewableInfo {
                entity:    id,
                force_new: camera.state.as_ref().is_some_and(|state| state.command),
            });

            if let Some(camera) = camera.state.as_ref().map(|state| state.camera) {
                focus_writer.write(gui::RequestCameraFocus { camera });
            }
        },
    );
}

pub trait ObservePicking {
    fn observe_picking(&mut self) -> &mut Self;
}

impl ObservePicking for EntityCommands<'_> {
    fn observe_picking(&mut self) -> &mut Self {
        add_observers(self);
        self
    }
}
