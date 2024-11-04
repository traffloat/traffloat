use std::collections::BTreeMap;

use bevy::app::{self, App};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::schedule::{IntoSystemConfigs, SystemSet};
use bevy::ecs::system::{Commands, Query, Res, ResMut, SystemParam};
use bevy_egui::egui;
use traffloat_base::{debug, ClientSideSystemSet};
use traffloat_view::metrics::{NewTypeMessage, RequestSubscribeMessage, UpdateMetricMessage};
use traffloat_view::viewer::{C2sMessageWriterSystemSet, S2cMessageReaderSystemSet};
use traffloat_view::{
    metrics as view_metrics, translation, viewable, C2sMessageEvent, C2sMessageWriter,
    S2cMessageReader,
};

use super::infobox;
use crate::util::glossary;
use crate::view::delegate;

pub(super) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<delegate::SidIndex<view_metrics::Sid>>();
        app.add_systems(
            app::Update,
            subscribe_new_metrics_system
                .in_set(S2cMessageReaderSystemSet::<NewTypeMessage>::default())
                .in_set(C2sMessageWriterSystemSet::<RequestSubscribeMessage>::default())
                .in_set(ClientSideSystemSet)
                .in_set(delegate::SidIndexMaintainerSystemSet::<view_metrics::Sid>::default()),
        );
        app.add_systems(
            app::Update,
            handle_metric_update_system
                .in_set(S2cMessageReaderSystemSet::<UpdateMetricMessage>::default())
                .before(ReceivedValuesReaderSystemSet)
                .in_set(ClientSideSystemSet)
                .after(delegate::SidIndexMaintainerSystemSet::<viewable::Sid>::default())
                .before(infobox::RenderSystemSet),
        );
    }
}

#[derive(Component)]
struct ReceivedValues(BTreeMap<view_metrics::Sid, f32>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
struct ReceivedValuesReaderSystemSet;

pub(super) fn object_bundle() -> impl Bundle { (ReceivedValues(BTreeMap::new()),) }

fn subscribe_new_metrics_system(
    mut reader: S2cMessageReader<NewTypeMessage>,
    mut sender: C2sMessageWriter<RequestSubscribeMessage>,
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
            message: RequestSubscribeMessage { ty: event.message.ty },
        });
    }
}

fn handle_metric_update_system(
    mut reader: S2cMessageReader<UpdateMetricMessage>,
    viewable_sid_index: Res<delegate::SidIndex<viewable::Sid>>,
    mut viewable_query: Query<&mut ReceivedValues, With<delegate::Marker<viewable::Sid>>>,
) {
    for ev in reader.read() {
        let Some(viewable_entity) = viewable_sid_index.get(ev.message.viewable) else { continue };
        let mut known =
            viewable_query.get_mut(viewable_entity).expect("sid_index refers to invalid viewable");
        known.0.insert(ev.message.ty, ev.message.magnitude);
    }
}

#[derive(SystemParam)]
pub(super) struct RenderUiParams<'w, 's> {
    object_query: Query<'w, 's, &'static ReceivedValues, With<delegate::Marker<viewable::Sid>>>,
    metric_query: Query<
        'w,
        's,
        &'static view_metrics::ClientTypeData,
        With<delegate::Marker<view_metrics::Sid>>,
    >,
    metric_sid_index:  Res<'w, delegate::SidIndex<view_metrics::Sid>>,
    glossary_provider: glossary::Provider<'w>,
}

pub(super) fn render_ui(
    ui: &mut egui::Ui,
    viewable_entity: Entity,
    RenderUiParams { object_query, metric_query, metric_sid_index, glossary_provider }: &RenderUiParams,
) {
    let Ok(object_values) = object_query.get(viewable_entity) else { return };

    for (&ty, &value) in &object_values.0 {
        let ty_label = if let Some(type_entity) = metric_sid_index.get(ty) {
            match metric_query.get(type_entity) {
                Ok(def) => def.quantified.render_to_string(
                    glossary_provider,
                    &[translation::Argument::Number(value.into())],
                ),
                Err(err) => {
                    bevy::log::warn!("metric SID has invalid metric delegate entity: {err:?}");
                    format!("{ty:?}")
                }
            }
        } else {
            bevy::log::warn!("object has invalid metric SID: {ty:?}");
            format!("{ty:?}")
        };

        ui.label(ty_label);
    }
}
