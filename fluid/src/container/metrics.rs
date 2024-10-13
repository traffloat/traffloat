use std::time::Duration;

use bevy::app::{self, App};
use bevy::ecs::event::EventWriter;
use bevy::ecs::query::{self, With};
use bevy::ecs::schedule::{IntoSystemConfigs, Schedules, SystemSet};
use bevy::ecs::system::Query;
use bevy::ecs::world::World;
use bevy::hierarchy;
use bevy::state::state::States;
use bevy::utils::HashMap;
use traffloat_base::partition;
use traffloat_view::{metrics, viewer};

use super::element;
use crate::config;

/// Maintains the state within each container.
pub(crate) struct Plugin<St>(pub(super) St);

impl<St: States + Copy> app::Plugin for Plugin<St> {
    fn build(&self, app: &mut App) {
        app.add_systems(config::OnCreateType, on_create_type_system.in_set(RegisterMetricType));
        app.add_systems(
            app::Update,
            on_new_viewer_system
                .in_set(partition::EventWriterSystemSet::<metrics::AvailableTypeEvent>::default()),
        );
    }
}

/// System set in which the metric type is registered for a new fluid type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct RegisterMetricType;

fn on_create_type_system(world: &mut World) {
    let fluid_type = world.resource::<config::CreatedType>().get();

    let metric_type = metrics::create_type(
        &mut world.commands(),
        metrics::TypeDef { update_frequency: Duration::from_secs(2) },
    );
    world.flush();

    world.entity_mut(fluid_type.0).insert(metric_type);

    let feeder = metrics::make_external_value_feeder_system::<
        (&config::Type, &hierarchy::Parent, &element::Mass),
        With<element::Marker>,
        With<super::Marker>,
        (),
        _,
    >(
        world,
        move |(&element_fluid_type, parent, mass), ()| {
            // TODO optimize this quadratic loop by
            // splitting fluid types into dynamic components on the facility.
            (fluid_type == element_fluid_type).then(|| (parent.get(), mass.mass.quantity))
        },
        metric_type,
    );
    let mut schedules = world.resource_mut::<Schedules>();
    schedules.add_systems(metrics::BroadcastSchedule, feeder);

    for viewer in world.query::<&viewer::Sid>().iter(world).copied().collect::<Vec<_>>() {
        world.send_event(metrics::AvailableTypeEvent {
            ty: metric_type,
            viewer,
            classes: HashMap::new(),
        });
    }
}

fn on_new_viewer_system(
    fluid_type_query: Query<&metrics::Type, With<config::TypeDef>>,
    viewer_query: Query<&viewer::Sid, query::Added<viewer::Sid>>,
    mut writer: EventWriter<metrics::AvailableTypeEvent>,
) {
    writer.send_batch(viewer_query.iter().flat_map(|&viewer| {
        fluid_type_query.iter().map(move |&ty| metrics::AvailableTypeEvent {
            viewer,
            ty,
            classes: HashMap::new(),
        })
    }));
}
