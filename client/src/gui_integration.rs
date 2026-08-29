//! Implements the interfaces required in [`traffloat_scene::gui`].

use std::iter;

use bevy::app::{self, App, Plugin};
use bevy::camera::{ImageRenderTarget, NormalizedRenderTarget};
use bevy::ecs::entity::Entity;
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Local, Query, Res, ResMut};
use bevy::math::Vec2;
use bevy::picking::mesh_picking::MeshPickingCamera;
use bevy::picking::pointer::{self, PointerAction, PointerId, PointerInput};
use bevy_egui::helpers::egui_vec2_into_vec2;
use egui_notify::Toast;
use either::Either;
use traffloat_scene::gui;

use crate::dock::{self, camera, plot, viewable_info};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.add_systems(app::Update, consume_scene_toasts_system);
        app.add_systems(app::Update, update_focus_system);
        app.add_systems(app::Update, camera_to_picking_system);
    }
}

fn consume_scene_toasts_system(
    mut toasts: ResMut<dock::Toasts>,
    mut reader: MessageReader<gui::ShowToast>,
) {
    for toast in reader.read() {
        toasts.0.add(match toast.level {
            gui::ToastLevel::Info => Toast::info(toast.message.clone()),
            gui::ToastLevel::Error => Toast::error(toast.message.clone()),
        });
    }
}

fn update_focus_system(dock: Res<dock::State>, commit: gui::UpdateFocusSystemParams) {
    commit.run(dock.tabs().flat_map(|tab| match tab {
        dock::TabEnum::ViewableInfo(tab) => {
            let iter = iter::once((tab.entity, gui::FocusClass::ViewInterior));
            Either::Left(Either::Left(iter))
        }
        dock::TabEnum::Plot(tab) => {
            let iter = tab
                .targets
                .iter()
                .flat_map(plot::Target::focused_entities)
                .map(|entity| (entity, gui::FocusClass::SubscribeData));
            Either::Left(Either::Right(iter))
        }
        _ => Either::Right(iter::empty()),
    }));
}

fn consume_open_viewable_info_system(
    mut reader: MessageReader<gui::OpenViewableInfo>,
    mut commands: Commands,
) {
    for request in reader.read() {
        commands.queue(viewable_info::OpenCommand {
            entity:    request.entity,
            force_new: request.force_new,
        });
    }
}

fn request_camera_focus_system(
    mut reader: MessageReader<gui::RequestCameraFocus>,
    mut dock: ResMut<dock::State>,
) {
    for request in reader.read() {
        dock.focus_tab(
            |tab| matches!(tab, dock::TabEnum::Camera(tab) if tab.camera == request.camera),
        );
    }
}

fn camera_to_picking_system(
    ui_state: Res<camera::UiState>,
    mut camera_interaction: ResMut<gui::CameraInteraction>,
    mut writer: MessageWriter<PointerInput>,
    camera_query: Query<Entity, With<MeshPickingCamera>>,
    mut commands: Commands,
    mut last_position: Local<Option<(Vec2, Entity)>>,
) {
    camera_interaction.state =
        ui_state.hover_state.as_ref().map(|state| gui::CameraInteractionState {
            camera:  state.camera,
            command: state.modifiers.command,
            shift:   state.modifiers.shift,
        });

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
