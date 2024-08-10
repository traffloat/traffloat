//! A viewable entity can be subscribed by a viewer.

use std::{iter, mem};

use bevy::app::{self, App};
use bevy::ecs::bundle;
use bevy::ecs::component::{Component, ComponentId};
use bevy::ecs::entity::{Entity, EntityHashSet};
use bevy::ecs::event::{Event, EventWriter, Events};
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Query, Res, ResMut, Resource};
use bevy::ecs::world::{DeferredWorld, World};
use bevy::math::bounding::Aabb3d;
use bevy::math::Vec3A;
use bevy::transform::components::Transform;
use bevy::utils::HashSet;
use derive_more::{From, Into};
use either::Either;
use kd_tree::KdTree3;
use typed_builder::TypedBuilder;

use crate::viewer;

/// Serialization-level identifier for this viewer.
///
/// This identifier is used for communication between server and client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Component, From, Into)]
pub struct Sid(u32);

/// Lookup server entities from `Sid`.
pub type SidIndex = super::IdIndex<Sid>;

/// Convenience method to allocate a new SID from the world.
pub fn next_sid(world: &mut World) -> Sid { world.resource_mut::<SidIndex>().next_id_mut() }

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        SidIndex::init(app.world_mut());

        app.add_event::<ShowEvent>();
        app.add_event::<HideEvent>();

        app.insert_resource(SpatialIndex { kdtree: None });
        app.add_systems(
            app::Update,
            (
                update_spatial_index_system,
                maintain_viewers_system.after(update_spatial_index_system),
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
    pub viewer:   viewer::Sid,
    /// The viewable to be showed.
    pub viewable: Sid,
}

/// The client should stop displaying a viewable.
#[derive(Debug, Event)]
pub struct HideEvent {
    /// The viewer to hide from.
    pub viewer:   viewer::Sid,
    /// The viewable to be hidden.
    pub viewable: Sid,
}

/// Extension omponents to construct a viewable entity.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    id:       Sid,
    #[builder(default)]
    viewers:  Viewers,
    position: Transform,
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
    query: Query<(Entity, &Transform), (With<Sid>, With<Static>)>,
) {
    if tree.kdtree.is_some() {
        return;
    }

    dbg!(query.iter().count());
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

/// A marker component to indicate that the viewer list of the component
/// is controlled by the view module.
///
/// Viewable entities without this component shall maintain [`Viewers`]
/// from the module that manages the viewable entity.
#[derive(Component, Default)]
pub struct Static;

fn maintain_viewers_system(
    tree: Res<SpatialIndex>,
    mut viewer_query: Query<(
        Entity,
        &viewer::Sid,
        &Transform,
        &viewer::Range,
        &mut viewer::ViewableList,
    )>,
    mut viewable_query: Query<(&Sid, &mut Viewers), With<Static>>,
    mut show_events: EventWriter<ShowEvent>,
    mut hide_events: EventWriter<HideEvent>,
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
            for (_, viewable) in next_viewable_vec {
                next_viewable_set.insert(*viewable);

                if prev_viewables.set.contains(viewable) {
                    continue;
                }

                let (&viewable_sid, mut viewers) = viewable_query
                    .get_mut(*viewable)
                    .expect("kvtree contains nonexistent viewable entity");
                let has_inserted = viewers.insert(viewer);
                assert!(
                    has_inserted,
                    "{viewer:?} exists in viewer list of {viewable:?} but {viewable:?} does not \
                     exist in viewable list of {viewer:?}"
                );

                show_events.send(ShowEvent { viewer: viewer_sid, viewable: viewable_sid });
            }

            for viewable in &prev_viewables.set {
                if next_viewable_set.contains(viewable) {
                    continue;
                }

                let (&viewable_sid, mut viewers) = viewable_query
                    .get_mut(*viewable)
                    .expect("kvtree contains nonexistent viewable entity");
                let has_removed = viewers.remove(viewer);
                assert!(
                    has_removed,
                    "{viewer:?} does not exist in viewer list of {viewable:?} but {viewable:?} \
                     exists in viewable list of {viewer:?}"
                );

                hide_events.send(HideEvent { viewer: viewer_sid, viewable: viewable_sid });
            }

            prev_viewables.set = next_viewable_set;
        },
    );
}
