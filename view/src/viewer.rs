//! A viewer entity represents an information subscriber that observes part of the word.

use bevy::app::{self, App};
use bevy::ecs::bundle;
use bevy::ecs::entity::Entity;
use bevy::ecs::world::World;
use bevy::prelude::Component;
use bevy::transform::components::Transform;
use bevy::utils::HashSet;
use derive_more::{From, Into};
use typed_builder::TypedBuilder;

/// Serialization-level identifier for this viewer.
///
/// This identifier is used for communication between server and client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Component, From, Into)]
pub struct Sid(u32);

/// Lookup server entities from `Sid`.
pub type SidIndex = super::IdIndex<Sid>;

/// Convenience method to allocate a new SID from the world.
pub fn next_sid(world: &mut World) -> Sid { world.resource_mut::<SidIndex>().next_id_mut() }

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) { SidIndex::init(app.world_mut()); }
}

/// Components for a viewer.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    position:      Transform,
    range:         Range,
    id:            Sid,
    #[builder(default, setter(skip))]
    last_viewable: ViewableList,
}

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
