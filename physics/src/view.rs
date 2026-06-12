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
use bevy::reflect::Reflect;
use bevy::time;
use bevy_mod_config::Config;
use enum_map::EnumMap;
use traffloat_proto::proto;

use crate::util::Throttle;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Viewer>();
        app.register_type::<NextProtoId>();

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

#[derive(Resource, Reflect)]
struct NextProtoId(proto::Id);

impl Default for NextProtoId {
    fn default() -> Self { NextProtoId(proto::Id(const { NonZeroU32::new(1).unwrap() })) }
}

#[derive(Component, Reflect, Default)]
pub struct Viewer {
    pub config: SubscriptionConfig,
}

impl Viewer {
    #[expect(
        clippy::unnecessary_wraps,
        reason = "will return None in the future with distance pruning"
    )]
    fn should_view(&self, viewable: &Viewable) -> Option<SubscriptionLevel> {
        // TODO distance pruning
        // TODO type filtering
        match self.config {
            SubscriptionConfig::Basic => Some(SubscriptionLevel::Basic),
            SubscriptionConfig::Full => Some(SubscriptionLevel::Full),
        }
    }

    #[must_use]
    pub fn with_level(mut self, level: SubscriptionConfig) -> Self {
        self.config = level;
        self
    }
    pub fn set_level(&mut self, level: impl Into<SubscriptionConfig>) {
        self.config = level.into();
    }
}

/// How much information a viewer wants to receive in general.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, enum_map::Enum, Config, Reflect)]
#[config(expose)]
pub enum SubscriptionConfig {
    #[default]
    Basic,
    Full,
}

/// How much information a viewer receives about a specific viewable.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, enum_map::Enum, Config, Reflect)]
#[config(expose)]
pub enum SubscriptionLevel {
    #[default]
    Basic,
    Full,
}

impl From<SubscriptionConfigRead> for SubscriptionConfig {
    fn from(value: SubscriptionConfigRead) -> Self {
        match value {
            SubscriptionConfigRead::Basic => SubscriptionConfig::Basic,
            SubscriptionConfigRead::Full => SubscriptionConfig::Full,
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

impl Viewable {
    #[must_use = "pass the result to MessageWrite::write_batch"]
    pub fn broadcast_update<Iter: IntoIterator<Item = proto::Update>>(
        &self,
        mut update: impl FnMut(SubscriptionLevel) -> Iter,
    ) -> impl Iterator<Item = SentUpdate> {
        self.subscribers.iter().filter(|(_, viewers)| !viewers.is_empty()).flat_map(
            move |(level, viewers)| {
                update(level).into_iter().map(|body| SentUpdate { viewers: viewers.clone(), body })
            },
        )
    }

    #[must_use = "pass the result to MessageWrite::write_batch"]
    pub fn broadcast_new<Iter: IntoIterator<Item = proto::Update>>(
        &self,
        update: impl FnOnce() -> Iter,
    ) -> impl Iterator<Item = SentUpdate> {
        (!self.new_subscribers.is_empty())
            .then(|| {
                let viewers: EntityHashSet = self.new_subscribers.iter().copied().collect();
                update().into_iter().map(move |body| SentUpdate { viewers: viewers.clone(), body })
            })
            .into_iter()
            .flatten()
    }
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

fn reconcile_subscription_system(
    viewer_query: Query<(Entity, &Viewer)>,
    mut viewable_query: Query<(Entity, &mut Viewable)>,
    mut messages: MessageWriter<'_, SentUpdate>,
) {
    for (_, mut viewable) in &mut viewable_query {
        viewable.new_subscribers.clear();

        let mut new_subscribers = EnumMap::<_, EntityHashSet>::default();
        for (viewer_entity, viewer) in &viewer_query {
            if let Some(level) = viewer.should_view(&viewable) {
                new_subscribers[level].insert(viewer_entity);
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
