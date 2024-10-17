use std::collections::BTreeMap;

use bevy::app::{self, App};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::{EventReader, EventWriter};
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Query, Res};
use bevy::hierarchy::ChildBuilder;
use bevy::text::{Text, TextSection, TextStyle};
use bevy::ui::node_bundles::TextBundle;
use traffloat_base::{debug, EventReaderSystemSet};
use traffloat_view::metrics as view_metrics;

use super::{DelegateSidIndex, DelegateViewable};

pub(super) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(app::Update, subscribe_new_metrics_system);
        app.add_systems(
            app::Update,
            handle_metric_update_system
                .in_set(EventReaderSystemSet::<view_metrics::UpdateMetricEvent>::default()),
        );
        app.add_systems(app::Update, update_text_system);
    }
}

#[derive(Component)]
struct Known(BTreeMap<view_metrics::Type, f32>);

pub(super) fn object_bundle() -> impl Bundle { (Known(BTreeMap::new()),) }

fn subscribe_new_metrics_system(
    mut reader: EventReader<view_metrics::NewTypeEvent>,
    mut sender: EventWriter<view_metrics::RequestSubscribeEvent>,
) {
    for ev in reader.read() {
        sender.send(view_metrics::RequestSubscribeEvent { viewer: ev.viewer, ty: ev.ty });
    }
}

fn handle_metric_update_system(
    mut reader: EventReader<view_metrics::UpdateMetricEvent>,
    sid_index: Res<DelegateSidIndex>,
    mut viewable_query: Query<&mut Known, With<DelegateViewable>>,
) {
    for ev in reader.read() {
        let Some(viewable_entity) = sid_index.get(ev.viewable) else { continue };
        let mut known =
            viewable_query.get_mut(viewable_entity).expect("sid_index refers to invalid viewable");
        known.0.insert(ev.ty, ev.magnitude);
    }
}

pub(super) fn spawn_ui(b: &mut ChildBuilder, viewable_entity: Entity) {
    b.spawn((
        TextBundle { text: Text::from_sections([]), ..Default::default() },
        ValueDisplay(viewable_entity),
        debug::Bundle::new("Infobox/Viewable/Metrics"),
    ));
}

/// Marks the entity as a UI text element for displaying the specified delegate viewable entity.
#[derive(Component)]
struct ValueDisplay(Entity);

fn update_text_system(
    mut display_query: Query<(&mut Text, &ValueDisplay)>,
    object_query: Query<&Known, With<DelegateViewable>>,
) {
    for (mut display, &ValueDisplay(viewable_entity)) in &mut display_query {
        let Ok(object_known) = object_query.get(viewable_entity) else { return };

        display.sections.clear();
        display.sections.extend(object_known.0.iter().map(|(ty, value)| {
            TextSection::new(
                format!("{ty:?}: {value}"),
                TextStyle { font_size: 16., ..Default::default() },
            )
        }));
    }
}
