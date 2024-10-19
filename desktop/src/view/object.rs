use bevy::app::{self, App};
use bevy::asset::AssetServer;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Res, ResMut};
use bevy::hierarchy::BuildChildren;
use bevy::prelude::SpatialBundle;
use bevy::render;
use bevy::transform::components::Transform;
use traffloat_base::debug;
use traffloat_view::viewer::S2cMessageReaderSystemSet;
use traffloat_view::{viewable, S2cMessageReader};

use super::delegate;

mod infobox;
mod layers;
mod metrics;

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<delegate::SidIndex<viewable::Sid>>();

        app.add_plugins((infobox::Plugin, layers::Plugin, metrics::Plugin));

        app.add_systems(
            app::Update,
            handle_show_system
                .in_set(S2cMessageReaderSystemSet::<viewable::ShowMessage>::default()),
        );
    }
}

fn handle_show_system(
    mut commands: Commands,
    mut reader: S2cMessageReader<viewable::ShowMessage>,
    mut viewable_sid_index: ResMut<delegate::SidIndex<viewable::Sid>>,
    assets: Res<AssetServer>,
) {
    for event in reader.read() {
        let viewable_id = viewable_sid_index.add(
            event.message.viewable,
            &mut commands,
            || {
                (
                    event.message.appearance.clone(),
                    SpatialBundle {
                        visibility: render::view::Visibility::Visible,
                        transform: Transform::from(event.message.transform),
                        ..Default::default()
                    },
                    infobox::object_bundle(),
                    metrics::object_bundle(),
                    debug::Bundle::new_with(|| {
                        format!(
                            "DelegateViewable({})",
                            event.message.appearance.label.short_debug()
                        )
                    }),
                )
            },
            |b| layers::spawn_all(b, &assets, &event.message),
        );

        commands.entity(viewable_id).insert(render::view::Visibility::Visible);
        if let Some(parent_sid) = event.message.parent {
            if let Some(parent_entity) = viewable_sid_index.get(parent_sid) {
                commands.entity(parent_entity).add_child(viewable_id);
            } else {
                bevy::log::warn!("received event with unknown parent id {parent_sid:?}");
            }
        }
    }
}
