//! GUI backend for the scene crate.
//!
//! This module is an abstraction of the user of the scene crate
//! providing UI interaction capability.
//! It does not do anything on its own.
//! Users are expected to consume the messages provided in this module.

use bevy::app::{App, Plugin};
use bevy::ecs::entity::{Entity, EntityHashSet};
use bevy::ecs::message::{Message, MessageWriter};
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Query, ResMut, SystemParam};
use traffloat_proto::proto;
use traffloat_util::QueryExt;

use crate::{OutboundRequest, ProtoId};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.add_message::<ShowToast>();
        app.add_message::<OpenViewableInfo>();
        app.add_message::<RequestCameraFocus>();
        app.init_resource::<CameraInteraction>();
        app.init_resource::<ViewInteriorEntities>();
    }
}

/// Show a toast notification.
#[derive(Message)]
pub struct ShowToast {
    pub level:   ToastLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLevel {
    Info,
    Error,
}

/// Update the set of focused entities.
///
/// This function should be called every tick to ensure that
/// the client receives update events for viewed entities promptly.
#[derive(SystemParam)]
pub struct UpdateFocusSystemParams<'w, 's> {
    id_query:               Query<'w, 's, &'static ProtoId>,
    writer:                 MessageWriter<'w, OutboundRequest>,
    view_interior_entities: ResMut<'w, ViewInteriorEntities>,
}

impl UpdateFocusSystemParams<'_, '_> {
    pub fn run(mut self, entities: impl IntoIterator<Item = (Entity, FocusClass)>) {
        self.view_interior_entities.set.clear();
        let mut focused_ids = Vec::new();

        for (entity, class) in entities {
            if class == FocusClass::ViewInterior {
                self.view_interior_entities.set.insert(entity);
            }
            if let Some(proto_id) = self.id_query.log_get(entity) {
                focused_ids.push(proto_id.0);
            }
        }

        self.writer.write(OutboundRequest {
            body: proto::Request::SetViewFocus(proto::SetViewFocus { focus: focused_ids }),
        });
    }
}

#[derive(Resource, Default)]
pub(crate) struct ViewInteriorEntities {
    pub(crate) set: EntityHashSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusClass {
    SubscribeData,
    ViewInterior,
}

/// Requests to open a viewable entity in the UI.
#[derive(Message)]
pub struct OpenViewableInfo {
    pub entity:    Entity,
    /// Whether the viewable entity should be opened in a new tab when supported.
    pub force_new: bool,
}

/// Requests to change focus to a specific camera.
#[derive(Message)]
pub struct RequestCameraFocus {
    pub camera: Entity,
}

/// The backend should *write* to this resource for camera interaction updates.
#[derive(Resource, Default)]
pub struct CameraInteraction {
    pub state: Option<CameraInteractionState>,
}

pub struct CameraInteractionState {
    pub camera:  Entity,
    pub command: bool,
    pub shift:   bool,
}
