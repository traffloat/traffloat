//! Calculates the sunlight level of each building

use std::cell::RefCell;
use std::cmp;
use std::collections::BTreeSet;
use std::f64::consts::PI;

use legion::Entity;

use crate::graph::*;
use crate::shape::Shape;
use crate::types::{Clock, Position, ScalarConfig};
use crate::SetupEcs;

/// The position of the sun
#[derive(Default)]
pub struct Sun {
    /// Orientation of the sun, in radians from +x towards +y
    pub yaw: f64,
}

#[legion::system]
fn move_sun(
    #[resource] sun: &mut Sun,
    #[resource] clock: &Clock,
    #[resource] config: &ScalarConfig,
) {
    sun.yaw += config.sun_speed * clock.delta;
}

/// Number of partitions to compute raytracing for
pub const MONTH_COUNT: usize = 12;

/// A component storing the lighting data for a node.
pub struct LightStats {
    /// The brightness values in each partition.
    ///
    /// The brightness value is the length receiving sunlight.
    pub brightness: [f64; MONTH_COUNT],
}

#[legion::system]
#[write_component(LightStats)]
#[read_component(Position)]
#[read_component(Shape)]
fn shadow_cast(
    world: &mut legion::world::SubWorld,
    #[state] first: &mut bool,
    #[state] node_add_sub: &mut shrev::ReaderId<NodeAddEvent>,
    #[resource] node_add_chan: &shrev::EventChannel<NodeAddEvent>,
    #[state] node_post_remove_sub: &mut shrev::ReaderId<PostNodeRemoveEvent>,
    #[resource] node_post_remove_chan: &shrev::EventChannel<PostNodeRemoveEvent>,
) {
    // we must drain the whole iterator even though we just want to know if there is at least one
    // item!
    let has_change = node_add_chan.read(node_add_sub).count() > 0
        && node_post_remove_chan.read(node_post_remove_sub).count() > 0;

    if !has_change && !*first {
        return;
    }
    *first = false;

    struct Marker<'t> {
        id: usize,
        x: f64,
        y: f64,
        start: Option<RefCell<&'t mut f64>>,
        entity: Entity,
    }
    impl<'t> PartialEq for Marker<'t> {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }
    impl<'t> Eq for Marker<'t> {}
    impl<'t> PartialOrd for Marker<'t> {
        fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
            self.x.partial_cmp(&other.x)
        }
    }
    impl<'t> Ord for Marker<'t> {
        fn cmp(&self, other: &Self) -> cmp::Ordering {
            self.partial_cmp(other).expect("infinite x")
        }
    }

    for month in 0..MONTH_COUNT {
        use legion::IntoQuery;

        #[allow(clippy::cast_precision_loss)]
        let angle = PI * 2. / (MONTH_COUNT as f64) * (month as f64);

        let mut query = <(Entity, &Position, &Shape, &mut LightStats)>::query();
        let marker_list = {
            let mut marker_list = Vec::new();
            for (&entity, &pos, shape, light) in query.iter_mut(&mut *world) {
                let transform = shape.transform(pos);
                nalgebra::Rotation2::<f64>::new(-angle);
                let (min, max) = shape.unit.bb_under(transform);

                let brightness = light
                    .brightness
                    .get_mut(month)
                    .expect("month is in iterator range");

                // take x max for all, y start and y end
                marker_list.push(Marker {
                    id: marker_list.len(),
                    x: max.x,
                    y: min.y,
                    start: Some(RefCell::new(brightness)),
                    entity,
                });
                marker_list.push(Marker {
                    id: marker_list.len(),
                    x: max.x,
                    y: max.y,
                    start: None,
                    entity,
                });
            }

            marker_list
                .sort_unstable_by(|a, b| a.y.partial_cmp(&b.y).expect("infinite bounding box"));

            marker_list
        };

        {
            #[allow(clippy::mutable_key_type)] // the RefCell<&mut f64> is not used for hashing
            let mut active_layers = BTreeSet::<&Marker<'_>>::new();
            let mut start = None;
            for marker in &marker_list {
                if let Some(start) = start {
                    if let Some(max) = active_layers.last() {
                        let brightness =
                            max.start.as_ref().expect("Only start markers are inserted");
                        let mut brightness = brightness.borrow_mut();
                        **brightness += marker.y - start;
                    }
                }

                if marker.start.is_some() {
                    active_layers.insert(marker);
                } else {
                    active_layers.remove(marker);
                }

                start = Some(marker.y);
            }
        };
    }
}

/// Initializes ECS
pub fn setup_ecs(mut setup: SetupEcs) -> SetupEcs {
    let node_add_event_sub = setup.subscribe::<NodeAddEvent>();
    let node_remove_event_sub = setup.subscribe::<PostNodeRemoveEvent>();
    setup
        .resource(Sun::default())
        .system(move_sun_system())
        .system(shadow_cast_system(
            true,
            node_add_event_sub,
            node_remove_event_sub,
        ))
}
