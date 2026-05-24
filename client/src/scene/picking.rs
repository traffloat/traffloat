use std::mem;

use bevy::app::{self, App, Plugin};
use bevy::camera::{ImageRenderTarget, NormalizedRenderTarget};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::ecs::observer;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, EntityCommands, Local, Query, Res};
use bevy::ecs::world::World;
use bevy::math::Vec2;
use bevy::picking::backend::PointerHits;
use bevy::picking::input::PointerInputSettings;
use bevy::picking::mesh_picking::{MeshPickingCamera, MeshPickingSettings};
use bevy::picking::pointer::{PointerAction, PointerId, PointerInput};
use bevy::picking::{PickingSettings, events as pick_event, pointer};
use bevy_egui::helpers::egui_vec2_into_vec2;

use crate::dock::{self, TabPlacement, camera, viewable_info};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.insert_resource(PickingSettings { is_input_enabled: true, ..Default::default() });
        app.insert_resource(PointerInputSettings { is_mouse_enabled: false, ..Default::default() });
        app.insert_resource(MeshPickingSettings { require_markers: true, ..Default::default() });
        app.add_systems(app::Update, input_system);
    }
}

fn input_system(
    ui_state: Res<camera::UiState>,
    mut writer: MessageWriter<PointerInput>,
    camera_query: Query<Entity, With<MeshPickingCamera>>,
    mut commands: Commands,
    mut last_position: Local<Option<(Vec2, Entity)>>,
) {
    let hovered_camera = ui_state.hover_state.as_ref().map(|state| state.camera);
    let mut has_marker = false;
    for camera in camera_query {
        if hovered_camera == Some(camera) {
            has_marker = true;
        } else {
            // Clear the hover state for cameras that are not hovered.
            commands.entity(camera).remove::<MeshPickingCamera>();
        }
    }
    if !has_marker && let Some(hovered_camera) = hovered_camera {
        commands.entity(hovered_camera).insert(MeshPickingCamera);
    }

    if let Some(ref hover) = ui_state.hover_state {
        let viewport_pos = egui_vec2_into_vec2(hover.viewport_pos);
        let last_position = last_position.replace((viewport_pos, hover.camera));
        let delta = if let Some((last_position, last_camera)) = last_position
            && last_camera == hover.camera
        {
            viewport_pos - last_position
        } else {
            Vec2::ZERO
        };

        let event = PointerInput {
            pointer_id: PointerId::Mouse,
            action:     PointerAction::Move { delta },
            location:   pointer::Location {
                target:   NormalizedRenderTarget::Image(ImageRenderTarget {
                    handle:       hover.image.clone(),
                    scale_factor: 1.0,
                }),
                position: egui_vec2_into_vec2(hover.viewport_pos),
            },
        };
        writer.write(event.clone());

        for (cond, action) in [
            (hover.primary_just_pressed, PointerAction::Press(pointer::PointerButton::Primary)),
            (hover.primary_just_released, PointerAction::Release(pointer::PointerButton::Primary)),
            (hover.secondary_just_pressed, PointerAction::Press(pointer::PointerButton::Secondary)),
            (
                hover.secondary_just_released,
                PointerAction::Release(pointer::PointerButton::Secondary),
            ),
        ] {
            if cond {
                let pointer_event = PointerInput { action, ..event.clone() };
                writer.write(pointer_event);
            }
        }
    }
}

#[derive(Component)]
pub struct Hovered;

pub fn add_observers(entity: &mut EntityCommands) {
    let id = entity.id();
    entity
        .observe(
            move |event: observer::On<pick_event::Pointer<pick_event::Over>>,
                  mut commands: Commands| {
                commands.entity(id).insert(Hovered);
            },
        )
        .observe(
            move |event: observer::On<pick_event::Pointer<pick_event::Out>>,
                  mut commands: Commands| {
                commands.entity(id).remove::<Hovered>();
            },
        )
        .observe(
            move |event: observer::On<pick_event::Pointer<pick_event::Click>>,
                  mut commands: Commands| {
                commands.queue(move |world: &mut World| {
                    world.resource_mut::<dock::State>().focus_or_create(
                        || viewable_info::Tab { entity: id }.into(),
                        dock::AfterTab(|state| matches!(state.tab, dock::TabEnum::ViewableInfo(_)))
                            .or_always(dock::SplitRoot {
                                split: egui_dock::Split::Right,
                                ratio: 0.7,
                            }),
                    );
                });
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
