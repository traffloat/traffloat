use std::collections::HashMap;
use std::num::NonZeroU32;
use std::time::Duration;
use std::{iter, mem};

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::{Entity, EntityHashMap, EntityHashSet};
use bevy::ecs::message::{Message, MessageWriter};
use bevy::ecs::query::{Has, With};
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::{IntoScheduleConfigs, SystemSet};
use bevy::ecs::system::{EntityCommand, Query, Res, SystemParam};
use bevy::ecs::world::EntityWorldMut;
use bevy::math::{Rect, Vec2};
use bevy::reflect::Reflect;
use bevy::time;
use enum_map::EnumMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use traffloat_proto::proto;

use crate::util::{self, QueryExt, Throttle};
use crate::{CleanupAppExt, request};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Viewer>();
        app.register_type::<Named>();
        app.register_type::<NextProtoId>();
        app.register_type::<IdIndex>();

        app.init_resource::<NextProtoId>();
        app.init_resource::<IdIndex>();
        app.add_message::<SentUpdate>();
        util::configure_enum_system_set::<SendUpdatesSystemSet>(app, app::Update);
        for set in SendUpdatesSystemSet::iter() {
            app.configure_sets(app::Update, set.before(traffloat_proto::UpdateHandlerSystemSet));
        }

        app.add_systems(
            app::Update,
            reconcile_subscription_system.in_set(SendUpdatesSystemSet::Pair),
        );
        app.add_cleanup_hook(|world| world.resource_mut::<IdIndex>().index.clear());
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter)]
pub enum SendUpdatesSystemSet {
    Meta,
    Cull,
    Pair,
    Init,
    Incr,
}

#[derive(Resource, Reflect)]
struct NextProtoId(proto::Id);

impl Default for NextProtoId {
    fn default() -> Self { NextProtoId(proto::Id(const { NonZeroU32::new(1).unwrap() })) }
}

#[derive(Resource, Reflect, Default)]
struct IdIndex {
    index: HashMap<proto::Id, Entity>,
}

#[derive(Component, Reflect, Default)]
pub struct Viewer {
    pub config:         SubscriptionConfig,
    pub viewports:      Vec<Rect>,
    pub focus_requests: Vec<proto::Id>,
}

impl Viewer {
    fn should_view(&self, viewable: &Viewable, viewable_bb: Rect) -> Option<SubscriptionLevel> {
        // TODO type filtering
        match self.config {
            SubscriptionConfig::Normal => {
                self.if_focused(viewable, SubscriptionLevel::Detail, viewable_bb)
            }
            SubscriptionConfig::Debug => {
                self.if_focused(viewable, SubscriptionLevel::Debug, viewable_bb)
            }
            SubscriptionConfig::Scraper => Some(SubscriptionLevel::Debug),
        }
    }

    fn if_focused(
        &self,
        viewable: &Viewable,
        level: SubscriptionLevel,
        viewable_bb: Rect,
    ) -> Option<SubscriptionLevel> {
        if self.focus_requests.contains(&viewable.id) {
            Some(level)
        } else if self.viewports.iter().any(|viewport| !viewport.intersect(viewable_bb).is_empty())
        {
            Some(SubscriptionLevel::Optical)
        } else {
            None
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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, enum_map::Enum, Reflect)]
pub enum SubscriptionConfig {
    #[default]
    Normal,
    Debug,
    Scraper,
}

/// How much information a viewer receives about a specific viewable.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    enum_map::Enum,
    strum::EnumIter,
    Serialize,
    Deserialize,
    Reflect,
)]
pub enum SubscriptionLevel {
    Optical,
    Detail,
    Debug,
}

impl SubscriptionLevel {
    pub fn to_proto_subscribed_by(&self) -> proto::SubscribedBy {
        match self {
            SubscriptionLevel::Optical => proto::SubscribedBy::OPTICAL,
            SubscriptionLevel::Detail => proto::SubscribedBy::DETAIL,
            SubscriptionLevel::Debug => proto::SubscribedBy::DEBUG,
        }
    }
}

#[derive(Component)]
#[require(CullingRect)]
pub struct Viewable {
    /// Set of [`Viewer`] entities to receive events about this entity.
    subscribers: EnumMap<SubscriptionLevel, EntityHashSet>,

    pub(crate) id: proto::Id,

    /// List of [`Viewer`] entities that have just subscribed to this entity,
    /// or changed in subscription level.
    ///
    /// Viewable providers should watch this list and send initial update messages to each new subscriber.
    new_subscribers: SortedSubscriptionChanges,
}

impl Viewable {
    /// Broadcast update to viewers, grouped by subscription level.
    #[must_use = "pass the result to MessageWrite::write_batch"]
    pub fn broadcast_update<Iter: IntoIterator<Item = proto::Update>>(
        &self,
        mut update: impl FnMut(SubscriptionLevel) -> Iter,
    ) -> impl Iterator<Item = SentUpdate> {
        self.subscribers.iter().filter(|(_, viewers)| !viewers.is_empty()).flat_map(
            move |(level, viewers)| {
                update(level).into_iter().map(|body| {
                    tracing::trace!(
                        "broadcast message {} to viewers of {:?}",
                        AsRef::<str>::as_ref(&body),
                        self.id
                    );
                    SentUpdate { viewers: viewers.clone(), body }
                })
            },
        )
    }

    /// Broadcast update to viewers receiving optical updates to all the given viewables
    /// and focuses on at least one of them.
    pub fn broadcast_update_if_all_optical_and_any_detail<'d, Iter>(
        viewables: impl IntoIterator<Item = &'d Viewable> + Clone,
        mut update: impl FnMut(SubscriptionLevel) -> Iter + Copy,
        new_subscribers_only: bool,
    ) -> impl Iterator<Item = SentUpdate>
    where
        Iter: IntoIterator<Item = proto::Update>,
    {
        let viewables = &viewables;
        let make_level_viewers = move |level| {
            let mut viewers = viewables
                .clone()
                .into_iter()
                .map(|viewable| &viewable.subscribers[level])
                .fold(EntityHashSet::new(), |mut union, set| {
                    union.extend(set.iter().copied());
                    union
                });

            for viewable in viewables.clone() {
                viewers
                    .retain(|viewer| viewable.subscribers.values().any(|set| set.contains(viewer)));
            }

            if new_subscribers_only {
                viewers.retain(|viewer| {
                    viewables
                        .clone()
                        .into_iter()
                        .any(|viewable| viewable.new_subscribers.entities().contains(viewer))
                });
            }

            viewers
        };

        let debug_viewers = make_level_viewers(SubscriptionLevel::Debug);

        let mut detail_viewers = make_level_viewers(SubscriptionLevel::Detail);
        if !debug_viewers.is_empty() {
            detail_viewers.retain(|viewer| !debug_viewers.contains(viewer));
        }

        let debug_updates = (!debug_viewers.is_empty()).then(move || {
            update(SubscriptionLevel::Debug).into_iter().map(move |body| {
                tracing::trace!(
                    "broadcast message {} to debug viewers",
                    AsRef::<str>::as_ref(&body),
                );
                SentUpdate { viewers: debug_viewers.clone(), body }
            })
        });
        let detail_updates = (!detail_viewers.is_empty()).then(move || {
            update(SubscriptionLevel::Detail).into_iter().map(move |body| {
                tracing::trace!(
                    "broadcast message {} to detail viewers",
                    AsRef::<str>::as_ref(&body),
                );
                SentUpdate { viewers: detail_viewers.clone(), body }
            })
        });

        debug_updates.into_iter().flatten().chain(detail_updates.into_iter().flatten())
    }

    /// Broadcast update to new subscribers who did not subscribe to this viewable previously.
    #[must_use = "pass the result to MessageWrite::write_batch"]
    pub fn broadcast_new<Iter: IntoIterator<Item = proto::Update>>(
        &self,
        mut update: impl FnMut() -> Iter,
    ) -> impl Iterator<Item = SentUpdate> {
        self.broadcast_new_or_changed(move |old, _| {
            old.is_none().then(&mut update).into_iter().flatten()
        })
    }

    /// Broadcast update to new subscribers who did not subscribe to this viewable previously,
    /// grouped by subscription level
    #[must_use = "pass the result to MessageWrite::write_batch"]
    pub fn broadcast_new_by_level<Iter: IntoIterator<Item = proto::Update>>(
        &self,
        update: impl Fn(SubscriptionLevel) -> Iter,
    ) -> impl Iterator<Item = SentUpdate> {
        self.broadcast_new_or_changed(move |old, new| {
            old.is_none().then(|| update(new)).into_iter().flatten()
        })
    }

    /// Broadcast update to subscribers who newly subscribed
    /// or changed to another subscription level.
    #[must_use = "pass the result to MessageWrite::write_batch"]
    pub fn broadcast_new_or_changed<Iter: IntoIterator<Item = proto::Update>>(
        &self,
        mut update: impl FnMut(Option<SubscriptionLevel>, SubscriptionLevel) -> Iter,
    ) -> impl Iterator<Item = SentUpdate> {
        self.new_subscribers.iter_by_change().flat_map(move |(change, viewers)| {
            update(change.old, change.new).into_iter().map(move |body| {
                tracing::trace!(
                    "broadcast message {} to incremental viewers of {:?}",
                    AsRef::<str>::as_ref(&body),
                    self.id
                );
                SentUpdate { viewers: viewers.clone(), body }
            })
        })
    }
}

/// Optional component on viewables that can be renamed.
#[derive(Component, Reflect)]
pub struct Named {
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct SubscriptionChange {
    old: Option<SubscriptionLevel>,
    new: SubscriptionLevel,
}

#[derive(Default)]
struct SortedSubscriptionChanges(Vec<(SubscriptionChange, Entity)>);

impl SortedSubscriptionChanges {
    fn new(mut changes: Vec<(SubscriptionChange, Entity)>) -> Self {
        changes.sort_by_key(|&(ch, _)| ch);
        Self(changes)
    }

    fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.0.iter().map(|&(_, entity)| entity)
    }

    fn iter_by_change(&self) -> impl Iterator<Item = (SubscriptionChange, EntityHashSet)> + '_ {
        let mut iter = self.0.iter().copied().peekable();
        iter::from_fn(move || {
            let &(change, _) = iter.peek()?;
            Some((
                change,
                iter.by_ref()
                    .take_while(|&(ch, _)| ch == change)
                    .map(|(_, entity)| entity)
                    .collect(),
            ))
        })
    }

    /// Clears the list and swap out the buffer to avoid reallocation
    fn take_capacity(&mut self) -> Vec<(SubscriptionChange, Entity)> {
        self.0.clear();
        mem::take(&mut self.0)
    }
}

/// Component on viewables,
/// indicating that any viewers with viewport intersecting with this rect
/// should optically subscribe to this viewable.
///
/// This is a required component on [`Viewable`].
/// If a viewable entity is spawned without this component,
/// this defaults to infinity.
#[derive(Debug, Clone, Copy, Component, Reflect)]
pub struct CullingRect(pub Rect);

impl Default for CullingRect {
    fn default() -> Self { CullingRect(Rect { min: Vec2::NEG_INFINITY, max: Vec2::INFINITY }) }
}

pub struct AddViewableCommand;

impl EntityCommand for AddViewableCommand {
    type Out = ();
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
            new_subscribers: SortedSubscriptionChanges::default(),
        });

        let entity_id = entity.id();
        entity.resource_mut::<IdIndex>().index.insert(id, entity_id);
    }
}

#[derive(SystemParam)]
pub(crate) struct ReconcileSubscriptionParams<'w, 's> {
    viewer_query:   Query<'w, 's, (Entity, &'static Viewer)>,
    viewable_query: Query<'w, 's, (Entity, &'static mut Viewable, &'static CullingRect)>,
    messages:       MessageWriter<'w, SentUpdate>,
}

fn reconcile_subscription_system(mut params: ReconcileSubscriptionParams) {
    // TODO optimize this with a spatial index

    let mut prev_sub_levels = EntityHashMap::new();

    for (viewable_entity, mut viewable, culling_rect) in params.viewable_query {
        let mut new_subscribers = viewable.new_subscribers.take_capacity();
        prev_sub_levels.clear();
        for (level, viewers) in &viewable.subscribers {
            for &viewer in viewers {
                prev_sub_levels.insert(viewer, level);
            }
        }

        for (viewer_entity, viewer) in params.viewer_query {
            if let Some(level) = viewer.should_view(&viewable, culling_rect.0) {
                match prev_sub_levels.remove(&viewer_entity) {
                    Some(prev_level) if prev_level == level => {
                        // still subscribed at the same level, do nothing
                    }
                    Some(prev_level) => {
                        // subscription level changed
                        viewable.subscribers[prev_level].remove(&viewer_entity);
                        viewable.subscribers[level].insert(viewer_entity);
                        new_subscribers.push((
                            SubscriptionChange { old: Some(prev_level), new: level },
                            viewer_entity,
                        ));
                    }
                    None => {
                        // new subscriber
                        viewable.subscribers[level].insert(viewer_entity);
                        new_subscribers
                            .push((SubscriptionChange { old: None, new: level }, viewer_entity));
                    }
                }
            }
        }

        viewable.new_subscribers = SortedSubscriptionChanges(new_subscribers);

        if !prev_sub_levels.is_empty() {
            for (viewer, level) in &prev_sub_levels {
                viewable.subscribers[*level].remove(viewer);
            }
            params.messages.write(SentUpdate {
                viewers: prev_sub_levels.keys().copied().collect(),
                body:    proto::Update::RemoveViewable(proto::RemoveViewable { id: viewable.id }),
            });
        }
    }
}

/// Call before despawning a viewable entity previously added with [`AddViewableCommand`].
/// Must be called before calling despawn.
pub fn before_viewable_despawn(entity: &mut EntityWorldMut) {
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

    entity.resource_mut::<IdIndex>().index.remove(&id);
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

#[derive(SystemParam)]
pub(crate) struct SetSubscriptionHandler<'w, 's> {
    viewer_query: Query<'w, 's, &'static mut Viewer>,
}

impl request::Handler for SetSubscriptionHandler<'_, '_> {
    type Request = proto::SetSubscription;

    fn classify(request: &Self::Request) -> request::HandlerClass { request::HandlerClass::Mutate }

    fn handle(&mut self, viewer: Entity, request: &Self::Request) {
        let Some(mut viewer) = self.viewer_query.log_get_mut(viewer) else { return };
        viewer.viewports.clone_from(&request.viewports);
        viewer.config =
            if request.debug { SubscriptionConfig::Debug } else { SubscriptionConfig::Normal };
    }
}

#[derive(SystemParam)]
pub(crate) struct SetViewFocusHandler<'w, 's> {
    viewer_query: Query<'w, 's, &'static mut Viewer>,
}

impl request::Handler for SetViewFocusHandler<'_, '_> {
    type Request = proto::SetViewFocus;

    fn classify(request: &Self::Request) -> request::HandlerClass { request::HandlerClass::Mutate }

    fn handle(&mut self, viewer: Entity, request: &Self::Request) {
        let Some(mut viewer) = self.viewer_query.log_get_mut(viewer) else { return };
        viewer.focus_requests.clone_from(&request.focus);
    }
}

#[derive(SystemParam)]
pub(crate) struct RenameViewableHandler<'w, 's> {
    viewable_query: Query<'w, 's, (Option<&'static mut Named>, &'static Viewable)>,
    index:          Res<'w, IdIndex>,
    update_writer:  MessageWriter<'w, SentUpdate>,
}

impl request::Handler for RenameViewableHandler<'_, '_> {
    type Request = proto::RenameViewable;

    fn classify(request: &Self::Request) -> request::HandlerClass { request::HandlerClass::Mutate }

    fn handle(&mut self, viewer: Entity, request: &Self::Request) {
        let viewable_entity = self
            .index
            .index
            .get(&request.id)
            .and_then(|&viewable_entity| self.viewable_query.log_get_mut(viewable_entity));
        match viewable_entity {
            None => {
                send_error_toast(
                    &mut self.update_writer,
                    viewer,
                    format!("Viewable with id {} not found", request.id.0),
                );
            }
            Some((None, _)) => {
                send_error_toast(
                    &mut self.update_writer,
                    viewer,
                    format!("Viewable with id {} is not renameable", request.id.0),
                );
            }
            Some((Some(mut named), viewable)) => {
                if request.name.is_empty() {
                    send_error_toast(&mut self.update_writer, viewer, "Name cannot be empty");
                    return;
                }

                named.name.clone_from(&request.name);
                self.update_writer.write_batch(viewable.broadcast_update(|_| {
                    [proto::UpdateViewableName { id: request.id, name: request.name.clone() }
                        .into()]
                }));
            }
        }
    }
}

pub fn send_error_toast(
    update_writer: &mut MessageWriter<SentUpdate>,
    viewer: Entity,
    message: impl Into<String>,
) {
    update_writer.write(SentUpdate {
        viewers: iter::once(viewer).collect(),
        body:    proto::ShowGenericToast {
            message: message.into(),
            ty:      proto::ToastType::Error,
        }
        .into(),
    });
}
