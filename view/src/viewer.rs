//! A viewer entity represents an information subscriber that observes part of the word.

use bevy::app::{self, App};
use bevy::ecs::bundle;
use bevy::ecs::entity::Entity;
use bevy::prelude::Component;
use bevy::transform::components::Transform;
use bevy::utils::HashSet;
use typed_builder::TypedBuilder;

sid_alias!("viewer");

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
