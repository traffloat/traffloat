//! A viewer entity represents an information subscriber that observes part of the word.

use bevy::app::{self, App};
use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::{Event, EventReader, EventWriter};
use bevy::transform::components::Transform;
use bevy::utils::HashSet;
use serde::de::DeserializeOwned;
use serde::Serialize;
use traffloat_base::{debug, EventReaderSystemSet, EventWriterSystemSet};
use typed_builder::TypedBuilder;

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, _app: &mut App) {}
}

/// Components for a viewer.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    position:      Transform,
    range:         Range,
    #[builder(default, setter(skip))]
    last_viewable: ViewableList,
    #[builder(default, setter(skip))]
    _marker:       Marker,
    #[builder(default = debug::Bundle::new("Viewer"))]
    _debug:        debug::Bundle,
}

/// Marks an entity as a viewer entity.
#[derive(Default, Component)]
pub struct Marker;

/// List of viewables displayed to the viewer.
#[derive(Component, Default)]
pub struct ViewableList {
    /// Set of viewable entities.
    pub set: HashSet<Entity>,
}

/// The maximum distance a viewer can observe.
///
/// Due to optimization concerns, the distance is interpreted as max-norm instead of 2-norm.
#[derive(Component)]
pub struct Range {
    /// The maximum distance a viewer can observe.
    pub distance: f32,
}

/// Dedicates a message from the server for a specific viewer.
#[derive(Event)]
pub struct S2cMessageEvent<E: S2cMessage> {
    /// The viewer to send to.
    pub viewer:  Entity,
    /// Message data.
    pub message: E,
}

/// Resource to write server-side messages.
pub type S2cMessageWriter<'w, E> = EventWriter<'w, S2cMessageEvent<E>>;

/// Systems sending messages of type E.
pub type S2cMessageWriterSystemSet<E> = EventWriterSystemSet<S2cMessageEvent<E>>;

/// Resource to read server-side messages.
pub type S2cMessageReader<'w, 's, E> = EventReader<'w, 's, S2cMessageEvent<E>>;

/// Systems receiving messages of type E.
pub type S2cMessageReaderSystemSet<E> = EventReaderSystemSet<S2cMessageEvent<E>>;

/// A message sent by the server.
pub trait S2cMessage: Serialize + DeserializeOwned {}

/// Contains a message from a viewer.
#[derive(Event)]
pub struct C2sMessageEvent<E: C2sMessage> {
    /// The viewer sending the message.
    pub viewer:  Entity,
    /// Message data.
    pub message: E,
}

/// Resource to write client-side messages.
pub type C2sMessageWriter<'w, E> = EventWriter<'w, C2sMessageEvent<E>>;

/// Systems sending messages of type E.
pub type C2sMessageWriterSystemSet<E> = EventWriterSystemSet<C2sMessageEvent<E>>;

/// Resource to read client-side messages.
pub type C2sMessageReader<'w, 's, E> = EventReader<'w, 's, C2sMessageEvent<E>>;

/// Systems handling messages of type E.
pub type C2sMessageReaderSystemSet<E> = EventReaderSystemSet<C2sMessageEvent<E>>;

/// A message sent by a viewer.
pub trait C2sMessage: Serialize + DeserializeOwned {}
