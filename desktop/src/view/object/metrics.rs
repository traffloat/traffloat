use std::collections::BTreeMap;

use bevy::app::{self, App};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Query, Res, ResMut};
use bevy::hierarchy::ChildBuilder;
use bevy::text::{Text, TextSection, TextStyle};
use bevy::ui::node_bundles::TextBundle;
use traffloat_base::debug;
use traffloat_view::viewer::{C2sMessageWriterSystemSet, S2cMessageReaderSystemSet};
use traffloat_view::{
    metrics as view_metrics, viewable, C2sMessageEvent, C2sMessageWriter, S2cMessageReader,
};

use crate::view::delegate;

pub(super) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<delegate::SidIndex<view_metrics::Sid>>();
        app.add_systems(
            app::Update,
            subscribe_new_metrics_system
                .in_set(S2cMessageReaderSystemSet::<view_metrics::NewTypeMessage>::default())
                .in_set(
                    C2sMessageWriterSystemSet::<view_metrics::RequestSubscribeMessage>::default(),
                ),
        );
        app.add_systems(
            app::Update,
            handle_metric_update_system
                .in_set(S2cMessageReaderSystemSet::<view_metrics::UpdateMetricMessage>::default()),
        );
        app.add_systems(app::Update, update_text_system);
    }
}

#[derive(Component)]
struct Known(BTreeMap<view_metrics::Sid, f32>);

pub(super) fn object_bundle() -> impl Bundle { (Known(BTreeMap::new()),) }

fn subscribe_new_metrics_system(
    mut reader: S2cMessageReader<view_metrics::NewTypeMessage>,
    mut sender: C2sMessageWriter<view_metrics::RequestSubscribeMessage>,
    mut commands: Commands,
    mut metrics_sid_index: ResMut<delegate::SidIndex<view_metrics::Sid>>,
) {
    for event in reader.read() {
        metrics_sid_index.add(
            event.message.ty,
            &mut commands,
            || (event.message.data.clone(), debug::Bundle::new("DelegateMetric")),
            |_| (),
        );
        sender.send(C2sMessageEvent {
            viewer:  event.viewer,
            message: view_metrics::RequestSubscribeMessage { ty: event.message.ty },
        });
    }
}

fn handle_metric_update_system(
    mut reader: S2cMessageReader<view_metrics::UpdateMetricMessage>,
    viewable_sid_index: Res<delegate::SidIndex<viewable::Sid>>,
    mut viewable_query: Query<&mut Known, With<delegate::Marker<viewable::Sid>>>,
) {
    for ev in reader.read() {
        let Some(viewable_entity) = viewable_sid_index.get(ev.message.viewable) else { continue };
        let mut known =
            viewable_query.get_mut(viewable_entity).expect("sid_index refers to invalid viewable");
        known.0.insert(ev.message.ty, ev.message.magnitude);
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
    object_query: Query<&Known, With<delegate::Marker<viewable::Sid>>>,
    metric_query: Query<&view_metrics::ClientTypeData, With<delegate::Marker<view_metrics::Sid>>>,
    metric_sid_index: Res<delegate::SidIndex<view_metrics::Sid>>,
) {
    for (mut display, &ValueDisplay(viewable_entity)) in &mut display_query {
        let Ok(object_known) = object_query.get(viewable_entity) else { return };

        display.sections.clear();
        display.sections.extend(object_known.0.iter().map(|(&ty, &value)| {
            let ty_label = if let Some(entity) = metric_sid_index.get(ty) {
                match metric_query.get(entity) {
                    Ok(def) => def.display_label.render_to_string(),
                    Err(err) => {
                        bevy::log::warn!("metric SID has invalid metric delegate entity: {err:?}");
                        format!("{ty:?}")
                    }
                }
            } else {
                bevy::log::warn!("object has invalid metric SID: {ty:?}");
                format!("{ty:?}")
            };
            TextSection::new(
                format!("{ty_label}: {value}\n"),
                TextStyle { font_size: 16., ..Default::default() },
            )
        }));
    }
}
