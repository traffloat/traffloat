use bevy::app::{self, App};
use bevy::asset::AssetServer;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::EventReader;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Res, ResMut, Resource};
use bevy::hierarchy::BuildChildren;
use bevy::prelude::SpatialBundle;
use bevy::render;
use bevy::transform::components::Transform;
use bevy::utils::HashMap;
use traffloat_base::{debug, EventReaderSystemSet};
use traffloat_view::viewable;

mod infobox;
mod layers;
mod metrics;

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DelegateSidIndex>();

        app.add_plugins((infobox::Plugin, layers::Plugin, metrics::Plugin));

        app.add_systems(
            app::Update,
            handle_show_system.in_set(EventReaderSystemSet::<viewable::ShowEvent>::default()),
        );
    }
}

/// Marks the entity as the delegate visualization for the SID.
#[derive(Debug, Clone, Copy, Component, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DelegateViewable(viewable::Sid);

// We cannot reuse the main SidIndex because it will overlap the entities
// if simulation is in the same world.
#[derive(Default, Resource)]
struct DelegateSidIndex(HashMap<viewable::Sid, Entity>);

fn handle_show_system(
    mut commands: Commands,
    mut reader: EventReader<viewable::ShowEvent>,
    mut sid_index: ResMut<DelegateSidIndex>,
    assets: Res<AssetServer>,
) {
    for ev in reader.read() {
        let viewable_id = *sid_index
            .0
            .entry(ev.viewable)
            .or_insert_with(|| spawn_object(&mut commands, ev, &assets));

        commands.entity(viewable_id).insert(render::view::Visibility::Visible);
        if let Some(parent_sid) = ev.parent {
            if let Some(&parent_entity) = sid_index.0.get(&parent_sid) {
                commands.entity(parent_entity).add_child(viewable_id);
            } else {
                bevy::log::warn!("received event with unknown parent id {parent_sid:?}");
            }
        }
    }
}

fn spawn_object(
    commands: &mut Commands,
    event: &viewable::ShowEvent,
    assets: &AssetServer,
) -> Entity {
    let mut parent = commands.spawn((
        DelegateViewable(event.viewable),
        event.appearance.clone(),
        SpatialBundle {
            visibility: render::view::Visibility::Visible,
            transform: Transform::from(event.transform),
            ..Default::default()
        },
        infobox::object_bundle(),
        metrics::object_bundle(),
        debug::Bundle::new("SubscribedObject"),
    ));
    layers::spawn_all(&mut parent, assets, event);

    parent.id()
}
