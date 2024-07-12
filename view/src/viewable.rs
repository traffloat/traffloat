//! A viewable entity can be subscribed by a viewer.

use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use smallvec::SmallVec;
use typed_builder::TypedBuilder;

/// Extension omponents to construct a viewable entity.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    #[builder(default = Viewers { viewers: <_>::default() })]
    viewers: Viewers,
}

/// Viewers of the viewable.
#[derive(Component)]
pub struct Viewers {
    pub viewers: SmallVec<[Entity; 4]>,
}
