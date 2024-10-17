use bevy::app::{self, App};
use bevy::asset::AssetServer;
use bevy::ecs::event::EventReader;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::hierarchy::BuildChildren;
use bevy::prelude::SpatialBundle;
use bevy::render;
use bevy::transform::components::Transform;
use traffloat_base::{debug, EventReaderSystemSet};
use traffloat_view::viewable;

use super::delegate;

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

type DelegateViewable = delegate::Marker<viewable::Sid>;

type DelegateSidIndex = delegate::SidIndex<viewable::Sid>;

fn handle_show_system(
    mut commands: Commands,
    mut reader: EventReader<viewable::ShowEvent>,
    mut sid_index: ResMut<DelegateSidIndex>,
    assets: Res<AssetServer>,
) {
    for event in reader.read() {
        let viewable_id = sid_index.add(
            event.viewable,
            &mut commands,
            || {
                (
                    event.appearance.clone(),
                    SpatialBundle {
                        visibility: render::view::Visibility::Visible,
                        transform: Transform::from(event.transform),
                        ..Default::default()
                    },
                    infobox::object_bundle(),
                    metrics::object_bundle(),
                    debug::Bundle::new("DelegateViewable"),
                )
            },
            |b| layers::spawn_all(b, &assets, event),
        );

        commands.entity(viewable_id).insert(render::view::Visibility::Visible);
        if let Some(parent_sid) = event.parent {
            if let Some(parent_entity) = sid_index.get(parent_sid) {
                commands.entity(parent_entity).add_child(viewable_id);
            } else {
                bevy::log::warn!("received event with unknown parent id {parent_sid:?}");
            }
        }
    }
}
