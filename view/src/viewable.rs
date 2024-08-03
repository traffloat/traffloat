//! A viewable entity can be subscribed by a viewer.

use std::mem;

use arrayvec::ArrayVec;
use bevy::app::{self, App};
use bevy::ecs::bundle;
use bevy::ecs::component::{Component, ComponentId};
use bevy::ecs::entity::Entity;
use bevy::ecs::event::{Event, EventWriter, Events};
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Query, Res, ResMut, Resource};
use bevy::ecs::world::DeferredWorld;
use bevy::math::bounding::Aabb3d;
use bevy::math::{Vec3, Vec3A};
use bevy::transform::components::Transform;
use kd_tree::KdTree3;
use smallvec::SmallVec;
use typed_builder::TypedBuilder;

use crate::viewer;

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
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
    pub viewer:   Entity,
    /// The viewable to be showed.
    pub viewable: Entity,
}

/// The client should stop displaying a viewable.
#[derive(Debug, Event)]
pub struct HideEvent {
    /// The viewer to hide from.
    pub viewer:   Entity,
    /// The viewable to be hidden.
    pub viewable: Entity,
}

/// Extension omponents to construct a viewable entity.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    #[builder(default = Viewers { viewers: <_>::default() })]
    viewers:  Viewers,
    position: Transform,
}

/// Viewers of the viewable.
#[derive(Component)]
pub struct Viewers {
    /// The list of viewers.
    pub viewers: SmallVec<[Entity; VIEWERS_SMALL_LEN]>,
}

const VIEWERS_SMALL_LEN: usize = 2;

fn init_viewers_for_viewable_hook(
    mut world: DeferredWorld,
    _entity: Entity,
    _comp_id: ComponentId,
) {
    world.resource_mut::<SpatialIndex>().kdtree = None;
}

fn update_spatial_index_system(
    mut tree: ResMut<SpatialIndex>,
    query: Query<(Entity, &Transform), With<Static>>,
) {
    if tree.kdtree.is_some() {
        return;
    }

    let viewables = query.iter().map(|(entity, tf)| (tf.translation.to_array(), entity)).collect();
    tree.kdtree = Some(KdTree3::build_by_ordered_float(viewables));
}

fn clean_viewers_hook(mut world: DeferredWorld, entity: Entity, _comp_id: ComponentId) {
    let viewers = {
        let mut viewers = world.get_mut::<Viewers>(entity).expect("entity in component hook");
        mem::take(&mut viewers.viewers)
    };

    world
        .resource_mut::<Events<HideEvent>>()
        .send_batch(viewers.iter().map(|&viewer| HideEvent { viewer, viewable: entity }));
}

#[derive(Resource)]
struct SpatialIndex {
    /// Position => viewable entity
    kdtree: Option<KdTree3<([f32; 3], Entity)>>,
}

/// Stores the last position for viewer maintenance.
#[derive(Component)]
struct LastPos(Vec3);

/// A marker component to indicate that the viewer list of the component
/// is controlled by the view module.
///
/// Viewable entities without this component shall maintain [`Viewers`]
/// from the module that manages the viewable entity.
#[derive(Component)]
pub struct Static;

fn maintain_viewers_system(
    mut commands: Commands,
    tree: Res<SpatialIndex>,
    viewer_query: Query<(Entity, &Transform, &viewer::Range, Option<&mut LastPos>)>,
    mut viewable_query: Query<&mut Viewers, With<Static>>,
    mut show_events: EventWriter<ShowEvent>,
    mut hide_events: EventWriter<HideEvent>,
) {
    let Some(kdtree) = &tree.kdtree else {
        return; // tree is currently inaccurate
    };

    viewer_query.iter().for_each(
        |(
            viewer,
            &Transform { translation: new_pos, .. },
            &viewer::Range { distance },
            last_pos,
        )| {
            match last_pos {
                None => {
                    commands.entity(viewer).insert(LastPos(new_pos));
                    // TODO benchmark if it is better to simply run this branch every time

                    let visible = Aabb3d::new(new_pos, Vec3A::splat(distance));
                    let viewables =
                        kdtree.within(&[visible.min.to_array(), visible.max.to_array()]);

                    for &&(_, viewable) in &viewables {
                        viewable_query
                            .get_mut(viewable)
                            .expect("kvtree contains nonexistent viewable entity")
                            .viewers
                            .push(viewer);
                    }

                    show_events.send_batch(
                        viewables.into_iter().map(|&(_, viewable)| ShowEvent { viewer, viewable }),
                    );
                }
                Some(&LastPos(last_pos)) => {
                    let last_aabb = Aabb3d::new(last_pos, Vec3A::splat(distance));
                    let new_aabb = Aabb3d::new(new_pos, Vec3A::splat(distance));

                    for bb in subtract_aabb(new_aabb, last_aabb) {
                        let new_viewables = kdtree.within(&[bb.min.to_array(), bb.max.to_array()]);
                        for &(_, viewable) in new_viewables {
                            let viewers = &mut viewable_query
                                .get_mut(viewable)
                                .expect("kvtree contains nonexistent viewable entity")
                                .viewers;
                            if !viewers.iter().any(|v| *v == viewable) {
                                viewers.push(viewer);
                                show_events.send(ShowEvent { viewer, viewable });
                            }
                        }
                    }

                    for bb in subtract_aabb(last_aabb, new_aabb) {
                        let new_viewables = kdtree.within(&[bb.min.to_array(), bb.max.to_array()]);
                        for &(_, viewable) in new_viewables {
                            let viewers = &mut viewable_query
                                .get_mut(viewable)
                                .expect("kvtree contains nonexistent viewable entity")
                                .viewers;
                            if let Some(index) = viewers.iter().position(|v| *v == viewable) {
                                viewers.swap_remove(index);
                                hide_events.send(HideEvent { viewer, viewable });
                            }
                        }
                    }
                }
            }
        },
    );
}

fn subtract_aabb(mut superset: Aabb3d, subset: Aabb3d) -> ArrayVec<Aabb3d, 6> {
    fn shrink(
        output: &mut ArrayVec<Aabb3d, 6>,
        superset: &mut Aabb3d,
        subset: &Aabb3d,
        field: impl Fn(Vec3A) -> f32,
        field_mut: impl Fn(&mut Vec3A) -> &mut f32,
    ) {
        if field(subset.max) < field(superset.max) {
            let partition = field(superset.min).max(field(subset.max));
            output.push({
                let mut diff = *superset;
                *field_mut(&mut diff.min) = partition;
                diff
            });
            *field_mut(&mut superset.max) = partition;
        }

        if field(subset.min) > field(superset.min) {
            let partition = field(superset.max).min(field(subset.min));
            output.push({
                let mut diff = *superset;
                *field_mut(&mut diff.max) = partition;
                diff
            });
            *field_mut(&mut superset.min) = partition;
        }
    }

    let mut output = ArrayVec::new();

    shrink(&mut output, &mut superset, &subset, |v| v.x, |v| &mut v.x);
    shrink(&mut output, &mut superset, &subset, |v| v.y, |v| &mut v.y);
    shrink(&mut output, &mut superset, &subset, |v| v.z, |v| &mut v.z);

    output
}
