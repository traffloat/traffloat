//! Calculates the sunlight level of each building

use std::cmp;
use std::collections::{btree_map::Entry, BTreeMap};
use std::convert::TryFrom;
use std::f64::consts::PI;

use smallvec::SmallVec;

use crate::config;
use crate::graph::*;
use crate::shape::Shape;
use crate::space::{Position, Vector};
use crate::time;
use crate::util::Finite;
use crate::SetupEcs;
use safety::Safety;

/// The position of the sun
#[derive(Default, getset::CopyGetters)]
pub struct Sun {
    /// Orientation of the sun, in radians from +x towards +y
    #[getset(get_copy = "pub")]
    yaw: f64,
}

impl Sun {
    /// Direction vector from any opaque object to the sun.
    pub fn direction(&self) -> Vector {
        Vector::new(self.yaw().cos(), self.yaw().sin())
    }
}

#[codegen::system]
fn move_sun(
    #[resource] sun: &mut Sun,
    #[resource] clock: &time::Clock,
    #[resource] config: &config::Scalar,
) {
    sun.yaw += config.sun_speed * clock.delta;
    sun.yaw %= PI * 2.;
}

/// Number of partitions to compute shadow casting for
pub const MONTH_COUNT: usize = 12;

/// A component storing the lighting data for a node.
#[derive(Debug, Default, getset::Getters)]
pub struct LightStats {
    /// The brightness values in each month.
    ///
    /// The brightness value is the area receiving sunlight.
    #[getset(get = "pub")]
    brightness: [f64; MONTH_COUNT],
}

#[codegen::system]
#[write_component(LightStats)]
#[read_component(Position)]
#[read_component(Shape)]
fn shadow_cast(
    world: &mut legion::world::SubWorld,
    #[state(true)] first: &mut bool,
    #[subscriber] node_additions: impl Iterator<Item = NodeAddEvent>,
    #[subscriber] node_post_removals: impl Iterator<Item = PostNodeRemoveEvent>,
) {
    // we must drain the whole iterator even though we just want to know if there is at least one
    // item!
    let has_change = node_additions.count() > 0 && node_post_removals.count() > 0;

    if !has_change && !*first {
        return;
    }
    *first = false;

    #[allow(clippy::indexing_slicing)]
    for month in 0..MONTH_COUNT {
        use legion::IntoQuery;
        let mut query = <(&mut LightStats, &Position, &Shape)>::query();

        struct Marker<'t> {
            light: &'t mut f64,
            min: Finite,
            max: Finite,
            priority: Finite,
        }

        let mut markers = Vec::new();

        for (stats, &position, shape) in query.iter_mut(world) {
            // Sun rotates from +x towards +y
            let yaw: f64 = PI * 2. / MONTH_COUNT.small_float::<f64>() * month.small_float::<f64>();

            // rot is the rotation matrix from the real coordinates to the time when yaw=0
            let rot = nalgebra::Rotation2::new(-yaw).to_homogeneous();
            let trans = shape.transform(position);
            let (min, max) = shape.unit().bb_under(rot * trans);

            let priority = Finite::new(max.x);
            let light = stats
                .brightness
                .get_mut(month)
                .expect("month < MONTH_COUNT");
            *light = 0.;

            let marker = Marker {
                light,
                min: Finite::new(min.y),
                max: Finite::new(max.y),
                priority,
            };
            markers.push(marker);
        }

        // The list of shadow edges
        let mut cuts: Vec<(Finite, usize)> = markers
            .iter()
            .enumerate()
            .map(|(i, marker)| (marker.min, i))
            .chain(
                markers
                    .iter()
                    .enumerate()
                    .map(|(i, marker)| (marker.max, i)),
            )
            .collect();
        cuts.sort_unstable();

        // If highest_list[i] == k, markers[k] is the highest marker in the grid
        // from y=cuts[i] to y=cuts[i + 1].
        // This is an intuitive algorithm similar to Carpenter's drawing.
        // The rare case that two markers have identical cuts
        // is handled by the discriminating index.
        let mut highest_list = vec![-1isize; cuts.len()];
        for (marker_index, marker) in markers.iter().enumerate() {
            let start: usize = cuts
                .binary_search(&(marker.min, marker_index))
                .expect("Marker not in cuts");
            let end: usize = cuts
                .binary_search(&(marker.max, marker_index))
                .expect("Marker not in cuts");

            // This is not right-inclusive,
            // because cuts[end]..cuts[end+1] is not covered.
            for highest in &mut highest_list[start..end] {
                let original = *highest;
                *highest = if original == -1 {
                    isize::try_from(marker_index).expect("too many markers")
                } else {
                    let ui = usize::try_from(original).expect("Converted from a usize");
                    if markers[ui].priority < marker.priority {
                        isize::try_from(marker_index).expect("too many markers")
                    } else {
                        original
                    }
                };
            }
        }

        for (i, &marker_index) in highest_list.iter().enumerate() {
            let len = cuts[i + 1].0.value() - cuts[i].0.value();
            if marker_index != -1 {
                let index = usize::try_from(marker_index).expect("Converted from a usize");
                let light = &mut *markers[index].light;
                *light += len;
            }
        }
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(move_sun_setup).uses(shadow_cast_setup)
}
