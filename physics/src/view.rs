use std::mem;
use std::num::NonZeroU32;
use std::time::Duration;

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::{Entity, EntityHashSet};
use bevy::ecs::message::{Message, MessageWriter};
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::{IntoScheduleConfigs, SystemSet};
use bevy::ecs::system::{EntityCommand, Query, SystemParam};
use bevy::ecs::world::EntityWorldMut;
use bevy::time;
use bevy_mod_config::Config;
use enum_map::EnumMap;
use traffloat_proto::proto;

use crate::util::Throttle;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextProtoId>();
        app.add_message::<SentUpdate>();
        app.add_systems(
            app::Update,
            reconcile_subscription_system.before(SendUpdatesSystemSet::Init),
        );
        app.configure_sets(
            app::Update,
            SendUpdatesSystemSet::Init.before(SendUpdatesSystemSet::Incr),
        );
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SendUpdatesSystemSet {
    Init,
    Incr,
}

#[derive(Resource)]
struct NextProtoId(proto::Id);

impl Default for NextProtoId {
    fn default() -> Self { NextProtoId(proto::Id(const { NonZeroU32::new(1).unwrap() })) }
}

#[derive(Component, Default)]
pub struct Viewer {
    level: SubscriptionLevel,
}

impl Viewer {
    fn should_view(&self, viewable: &Viewable) -> bool {
        // TODO distance pruning
        // TODO type filtering
        true
    }

    #[must_use]
    pub fn with_level(mut self, level: SubscriptionLevel) -> Self {
        self.level = level;
        self
    }
    pub fn set_level(&mut self, level: impl Into<SubscriptionLevel>) { self.level = level.into(); }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, enum_map::Enum, Config)]
#[config(expose)]
pub enum SubscriptionLevel {
    #[default]
    Basic,
    Full,
}

impl From<SubscriptionLevelRead> for SubscriptionLevel {
    fn from(value: SubscriptionLevelRead) -> Self {
        match value {
            SubscriptionLevelRead::Basic => SubscriptionLevel::Basic,
            SubscriptionLevelRead::Full => SubscriptionLevel::Full,
        }
    }
}

#[derive(Component)]
pub struct Viewable {
    /// Set of [`Viewer`] entities to receive events about this entity.
    pub subscribers: EnumMap<SubscriptionLevel, EntityHashSet>,

    pub(crate) id: proto::Id,

    /// List of [`Viewer`] entities that have just subscribed to this entity.
    ///
    /// Viewable providers should watch this list and send initial update messages to each new subscriber.
    pub new_subscribers: Vec<Entity>,
}

pub struct AddViewableCommand;

impl EntityCommand for AddViewableCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        let id = entity.world_scope(|world| {
            let mut next = world.resource_mut::<NextProtoId>();
            let id = next.0;
            next.0.0 = id.0.checked_add(1).expect("too many viewable entities");
            id
        });

        entity.insert(Viewable {
            subscribers: EnumMap::default(),
            id,
            new_subscribers: Vec::new(),
        });
    }
}

impl Viewable {
    pub fn broadcast_update(
        &self,
        mut update: impl FnMut(SubscriptionLevel) -> proto::Update,
    ) -> impl Iterator<Item = SentUpdate> {
        self.subscribers.iter().filter(|(_, viewers)| !viewers.is_empty()).map(
            move |(level, viewers)| SentUpdate { viewers: viewers.clone(), body: update(level) },
        )
    }
}

fn reconcile_subscription_system(
    viewer_query: Query<(Entity, &Viewer)>,
    mut viewable_query: Query<(Entity, &mut Viewable)>,
    mut messages: MessageWriter<'_, SentUpdate>,
) {
    for (_, mut viewable) in &mut viewable_query {
        viewable.new_subscribers.clear();

        let mut new_subscribers = EnumMap::<_, EntityHashSet>::default();
        for (viewer_entity, viewer) in &viewer_query {
            if viewer.should_view(&viewable) {
                new_subscribers[viewer.level].insert(viewer_entity);
            }
        }

        let old_subscribers = mem::replace(&mut viewable.subscribers, new_subscribers);
        let removals: EntityHashSet = old_subscribers
            .iter()
            .flat_map(|(_, s)| s)
            .copied()
            .filter(|e| !viewable.subscribers.iter().any(|(_, s)| s.contains(e)))
            .collect();
        if !removals.is_empty() {
            messages.write(SentUpdate {
                viewers: removals,
                body:    proto::Update::RemoveViewable(proto::RemoveViewable { id: viewable.id }),
            });
        }

        viewable.new_subscribers = viewable
            .subscribers
            .iter()
            .flat_map(|(_, s)| s)
            .copied()
            .filter(|e| !old_subscribers.iter().any(|(_, s)| s.contains(e)))
            .collect();
    }
}

pub fn on_viewable_despawn(entity: &mut EntityWorldMut) {
    let Some(viewable) = entity.get::<Viewable>() else { return };
    let viewers: EntityHashSet =
        viewable.subscribers.iter().flat_map(|(_, s)| s).copied().collect();
    let id = viewable.id;
    if !viewers.is_empty() {
        entity.world_scope(|world| {
            world.write_message(SentUpdate {
                viewers,
                body: proto::Update::RemoveViewable(proto::RemoveViewable { id }),
            });
        });
    }
}

#[derive(Message)]
pub struct SentUpdate {
    pub viewers: EntityHashSet,
    pub body:    proto::Update,
}

#[derive(SystemParam)]
pub struct BroadcastThrottle<'w, 's> {
    throttle: Throttle<'w, 's, time::Virtual>,
}

impl BroadcastThrottle<'_, '_> {
    pub fn should_run(&mut self) -> bool { self.throttle.should_run(Duration::from_millis(250)) }
}
