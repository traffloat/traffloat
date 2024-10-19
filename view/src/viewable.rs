//! A viewable entity can be subscribed by a viewer.

use std::{iter, mem};

use bevy::app::{self, App};
use bevy::ecs::bundle;
use bevy::ecs::component::{Component, ComponentId};
use bevy::ecs::entity::{Entity, EntityHashSet};
use bevy::ecs::event::{Event, EventReader, EventWriter, Events};
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Query, Res, ResMut, Resource};
use bevy::ecs::world::DeferredWorld;
use bevy::hierarchy;
use bevy::math::bounding::Aabb3d;
use bevy::math::Vec3A;
use bevy::transform::components::Transform;
use bevy::utils::HashSet;
use either::Either;
use kd_tree::KdTree3;
use traffloat_base::partition::{AppExt, EventReaderSystemSet, EventWriterSystemSet};
use traffloat_base::proto;
use typed_builder::TypedBuilder;

use crate::{appearance, viewer};

sid_alias!("viewable");

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        SidIndex::init(app.world_mut());

        app.add_partitioned_event::<ShowEvent>();
        app.add_partitioned_event::<ShowStationaryEvent>();
        app.add_partitioned_event::<HideEvent>();
        app.add_partitioned_event::<HideStationaryEvent>();

        app.insert_resource(SpatialIndex { kdtree: None });
        app.add_systems(
            app::Update,
            (
                update_spatial_index_system,
                update_stationary_viewers_system
                    .after(update_spatial_index_system)
                    .in_set(EventWriterSystemSet::<ShowEvent>::default())
                    .in_set(EventWriterSystemSet::<ShowStationaryEvent>::default())
                    .in_set(EventWriterSystemSet::<HideEvent>::default())
                    .in_set(EventWriterSystemSet::<HideStationaryEvent>::default()),
                (
                    show_stationary_children_system
                        .in_set(EventWriterSystemSet::<ShowEvent>::default())
                        .in_set(EventReaderSystemSet::<ShowStationaryEvent>::default()),
                    hide_stationary_children_system
                        .in_set(EventWriterSystemSet::<HideEvent>::default())
                        .in_set(EventReaderSystemSet::<HideStationaryEvent>::default()),
                )
                    .after(update_stationary_viewers_system),
            ),
        );
        app.world_mut()
            .register_component_hooks::<Viewers>()
            .on_add(init_viewers_for_viewable_hook);
        app.world_mut().register_component_hooks::<Viewers>().on_remove(clean_viewers_hook);
    }
}

/// The client should start displaying a viewable.
#[derive(Debug, Event)]
pub struct ShowEvent {
    /// The viewer to show to.
    pub viewer:     viewer::Sid,
    /// The viewable to be showed.
    pub viewable:   Sid,
    /// The parent viewable to display this object with.
    pub parent:     Option<Sid>,
    /// The model of the viewable.
    pub appearance: appearance::Appearance,
    /// The transform for the viewable model, relative to parent or world origin.
    pub transform:  proto::Transform,
}

/// A specialized `ShowEvent` that only gets sent for stationary viewables,
/// when updated by the stationary maintenance system.
#[derive(Debug, Event)]
pub struct ShowStationaryEvent {
    /// The viewer to show to.
    pub viewer:   Entity,
    /// The stationary viewable to be showed.
    pub viewable: Entity,
}

/// A specialized `HideEvent` that only gets sent for stationary viewables,
/// when updated by the stationary maintenance system.
#[derive(Debug, Event)]
pub struct HideStationaryEvent {
    /// The viewer to hide to.
    pub viewer:   Entity,
    /// The stationary viewable to be hideed.
    pub viewable: Entity,
}

/// The client should stop displaying a viewable.
#[derive(Debug, Event)]
pub struct HideEvent {
    /// The viewer to hide from.
    pub viewer:   viewer::Sid,
    /// The viewable to be hidden.
    pub viewable: Sid,
}

/// Common components to construct a viewable entity.
///
/// Entities should be initialized through [`StationaryBundle`] or [`StationaryChildBundle`] instead.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct BaseBundle {
    sid:        Sid,
    appearance: appearance::Appearance,
    #[builder(default)]
    viewers:    Viewers,
}

/// Initializes a viewable
#[derive(bundle::Bundle, TypedBuilder)]
pub struct StationaryBundle {
    /// Base components for a viewable.
    base:      BaseBundle,
    #[builder(default, setter(skip))]
    _marker:   Stationary,
    /// Absolute position vector from origin,
    /// along with absolute scaling and rotation for rendering.
    transform: Transform,
}

/// Initializes a viewable that is a [child](bevy::hierarchy::Children)
/// of a [stationary](Stationary) viewable.
///
/// Systems may panic if [`StationaryChild`] is applied on
/// an entity that is not a child of a stationary viewable.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct StationaryChildBundle {
    /// Base components for a viewable.
    base:            BaseBundle,
    #[builder(default, setter(skip))]
    _marker:         StationaryChild,
    /// Position of the viewable relative to its stationary parent.
    inner_transform: Transform,
}

/// Viewers of the viewable.
#[derive(Component)]
pub struct Viewers(ViewersInner);

enum ViewersInner {
    Array([Option<Entity>; VIEWERS_SMALL_LEN as usize]),
    HashSet(EntityHashSet),
}

impl Default for Viewers {
    fn default() -> Self { Self(ViewersInner::Array([None; VIEWERS_SMALL_LEN as usize])) }
}

impl Viewers {
    /// Insert a viewer entity.
    pub fn insert(&mut self, entity: Entity) -> bool {
        match self.0 {
            ViewersInner::Array(ref mut array) => {
                let mut gap = None;

                for item in &mut *array {
                    if *item == Some(entity) {
                        return false;
                    }

                    if item.is_none() {
                        gap = Some(item);
                    }
                }

                match gap {
                    Some(gap) => *gap = Some(entity),
                    None => {
                        self.0 = ViewersInner::HashSet(
                            array
                                .iter_mut()
                                .map(|option| option.expect("array is full"))
                                .chain(iter::once(entity))
                                .collect(),
                        );
                    }
                }

                true
            }
            ViewersInner::HashSet(ref mut set) => set.insert(entity),
        }
    }

    /// Checks if the entity is a viewer in this list.
    #[must_use]
    pub fn contains(&self, entity: Entity) -> bool {
        match self.0 {
            ViewersInner::Array(ref array) => array.iter().any(|item| *item == Some(entity)),
            ViewersInner::HashSet(ref set) => set.contains(&entity),
        }
    }

    /// Removes a viewer entity from the list.
    pub fn remove(&mut self, entity: Entity) -> bool {
        match self.0 {
            ViewersInner::Array(ref mut array) => {
                for item in array {
                    if *item == Some(entity) {
                        *item = None;
                        return true;
                    }
                }

                false
            }
            ViewersInner::HashSet(ref mut set) => {
                // no need to reallocate to `Array`
                set.remove(&entity)
            }
        }
    }

    /// Iterates over all entities in the viewer list.
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        match self.0 {
            ViewersInner::Array(array) => Either::Left(array.into_iter().flatten()),
            ViewersInner::HashSet(ref set) => Either::Right(set.iter().copied()),
        }
    }
}

const VIEWERS_SMALL_LEN: u32 = 2;

fn init_viewers_for_viewable_hook(
    mut world: DeferredWorld,
    _entity: Entity,
    _comp_id: ComponentId,
) {
    world.resource_mut::<SpatialIndex>().kdtree = None;
}

fn update_spatial_index_system(
    mut tree: ResMut<SpatialIndex>,
    query: Query<(Entity, &Transform), (With<Sid>, With<Stationary>)>,
) {
    if tree.kdtree.is_some() {
        return;
    }

    let viewables = query.iter().map(|(entity, tf)| (tf.translation.to_array(), entity)).collect();
    tree.kdtree = Some(KdTree3::build_by_ordered_float(viewables));
}

fn clean_viewers_hook(mut world: DeferredWorld, entity: Entity, _comp_id: ComponentId) {
    let &viewable_sid = world
        .get::<Sid>(entity)
        .expect("Viewers can only be in viewable entities with viewable::Sid");

    let viewers = {
        let mut viewers = world.get_mut::<Viewers>(entity).expect("entity in component hook");
        mem::take(&mut *viewers)
    };

    let hide_events: Vec<_> = viewers
        .iter()
        .map(|viewer| {
            let &viewer_sid = world
                .get::<viewer::Sid>(viewer)
                .expect("viewer list must reference valid viewer with viewer::Sid");
            HideEvent { viewer: viewer_sid, viewable: viewable_sid }
        })
        .collect();
    world.resource_mut::<Events<HideEvent>>().send_batch(hide_events);
}

#[derive(Resource)]
struct SpatialIndex {
    /// Position => viewable entity
    kdtree: Option<KdTree3<([f32; 3], Entity)>>,
}

/// A marker component to indicate that
/// the viewer list of the viewable entity is controlled by the view module
/// and the viewable entity has a stationary position.
///
/// Viewable entities without this component shall maintain [`Viewers`]
/// from the module that manages the viewable entity.
#[derive(Component, Default)]
pub struct Stationary;

/// A marker component to indicate that the
/// the viewer list of the viewable entity is controlled by the view module
/// and the viewable entity is a direct [child](bevy::hierarchy::Children)
/// of a [stationary](Stationary) viewable.
///
/// Viewable entities without this component shall maintain [`Viewers`]
/// from the module that manages the viewable entity.
#[derive(Component, Default)]
pub struct StationaryChild;

fn update_stationary_viewers_system(
    tree: Res<SpatialIndex>,
    mut viewer_query: Query<(
        Entity,
        &viewer::Sid,
        &Transform,
        &viewer::Range,
        &mut viewer::ViewableList,
    )>,
    mut viewable_query: Query<
        (&Sid, &appearance::Appearance, &Transform, &mut Viewers),
        With<Stationary>,
    >,
    mut show_events: EventWriter<ShowEvent>,
    mut show_stationary_events: EventWriter<ShowStationaryEvent>,
    mut hide_events: EventWriter<HideEvent>,
    mut hide_stationary_events: EventWriter<HideStationaryEvent>,
) {
    let Some(kdtree) = &tree.kdtree else {
        return; // tree is currently inaccurate
    };

    viewer_query.iter_mut().for_each(
        |(
            viewer,
            &viewer_sid,
            &Transform { translation: new_pos, .. },
            &viewer::Range { distance },
            mut prev_viewables,
        )| {
            let visible_aabb = Aabb3d::new(new_pos, Vec3A::splat(distance));
            let next_viewable_vec =
                kdtree.within(&[visible_aabb.min.to_array(), visible_aabb.max.to_array()]);

            let mut next_viewable_set = HashSet::with_capacity(next_viewable_vec.len());
            for &(_, viewable) in next_viewable_vec {
                next_viewable_set.insert(viewable);

                if prev_viewables.set.contains(&viewable) {
                    continue;
                }

                let (&viewable_sid, viewable_appearance, &viewable_transform, mut viewers) =
                    viewable_query
                        .get_mut(viewable)
                        .expect("kvtree contains nonexistent viewable entity");
                let has_inserted = viewers.insert(viewer);
                assert!(
                    has_inserted,
                    "{viewer:?} exists in viewer list of {viewable:?} but {viewable:?} does not \
                     exist in viewable list of {viewer:?}"
                );

                let show_event = ShowEvent {
                    viewer:     viewer_sid,
                    viewable:   viewable_sid,
                    parent:     None,
                    appearance: viewable_appearance.clone(),
                    transform:  viewable_transform.into(),
                };
                show_events.send(show_event);
                show_stationary_events.send(ShowStationaryEvent { viewer, viewable });
            }

            for viewable in &prev_viewables.set {
                if next_viewable_set.contains(viewable) {
                    continue;
                }

                let (&viewable_sid, _, _, mut viewers) = viewable_query
                    .get_mut(*viewable)
                    .expect("kvtree contains nonexistent viewable entity");
                let has_removed = viewers.remove(viewer);
                assert!(
                    has_removed,
                    "{viewer:?} does not exist in viewer list of {viewable:?} but {viewable:?} \
                     exists in viewable list of {viewer:?}"
                );

                hide_events.send(HideEvent { viewer: viewer_sid, viewable: viewable_sid });
                hide_stationary_events.send(HideStationaryEvent { viewer, viewable: *viewable });
            }

            prev_viewables.set = next_viewable_set;
        },
    );
}

fn show_stationary_children_system(
    mut show_stationary_events: EventReader<ShowStationaryEvent>,
    mut show_events: EventWriter<ShowEvent>,
    stationary_query: Query<(&Sid, &hierarchy::Children), With<Stationary>>,
    mut child_query: Query<
        (&Sid, &appearance::Appearance, &Transform, &mut Viewers),
        With<StationaryChild>,
    >,
    viewer_query: Query<&viewer::Sid>,
) {
    let mut events = Vec::new();
    for &ShowStationaryEvent { viewer: viewer_entity, viewable } in show_stationary_events.read() {
        let Ok((&stationary_sid, children)) = stationary_query.get(viewable) else {
            continue;
        };
        let &viewer_sid = viewer_query
            .get(viewer_entity)
            .expect("ShowStationaryEvent contains non-viewer viewer entity");

        for &child_entity in children {
            let Ok((&child_sid, child_appearance, &inner_transform, mut viewers)) =
                child_query.get_mut(child_entity)
            else {
                continue;
            };
            viewers.insert(viewer_entity);
            events.push(ShowEvent {
                viewer:     viewer_sid,
                viewable:   child_sid,
                parent:     Some(stationary_sid),
                appearance: child_appearance.clone(),
                transform:  inner_transform.into(),
            });
        }
    }

    show_events.send_batch(events);
}

fn hide_stationary_children_system(
    mut hide_stationary_events: EventReader<HideStationaryEvent>,
    mut hide_events: EventWriter<HideEvent>,
    stationary_query: Query<&hierarchy::Children, With<Stationary>>,
    mut child_query: Query<(&Sid, &mut Viewers), With<StationaryChild>>,
    viewer_query: Query<&viewer::Sid>,
) {
    let mut events = Vec::new();
    for &HideStationaryEvent { viewer: viewer_entity, viewable } in hide_stationary_events.read() {
        let Ok(children) = stationary_query.get(viewable) else { continue };
        let &viewer_sid = viewer_query
            .get(viewer_entity)
            .expect("HideStationaryEvent contains non-viewer viewer entity");

        for &child_entity in children {
            let Ok((&child_sid, mut viewers)) = child_query.get_mut(child_entity) else { continue };
            viewers.remove(viewer_entity);
            events.push(HideEvent { viewer: viewer_sid, viewable: child_sid });
        }
    }

    hide_events.send_batch(events);
}
