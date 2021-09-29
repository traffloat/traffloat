//! Calculates the sunlight level of each building

use std::cell::Cell;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::f64::consts::PI;

use smallvec::SmallVec;

use crate::appearance::Appearance;
use crate::clock::Clock;
use crate::space::{Position, Vector};
use crate::units::Brightness;
use crate::util::Finite;
use crate::{config, node, SetupEcs};
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
        Vector::new(self.yaw().cos(), self.yaw().sin(), 0.)
    }
}

#[codegen::system(Schedule)]
fn move_sun(
    #[resource] sun: &mut Sun,
    #[resource] clock: &Clock,
    #[resource] config: &config::Scalar,

    #[debug("Time", "Clock")] clock_debug: &codegen::DebugEntry,
    #[debug("Time", "Sun")] sun_debug: &codegen::DebugEntry,
) {
    sun.yaw += config.sun_speed * clock.delta();
    sun.yaw %= PI * 2.;

    codegen::update_debug!(clock_debug, "{}", clock.now().since_epoch().value());
    codegen::update_debug!(
        sun_debug,
        "({:.1}, {:.1}, {:.1})",
        sun.direction().x,
        sun.direction().y,
        sun.direction().z,
    );
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
    brightness: [Brightness; MONTH_COUNT],
}

#[codegen::system(PostCommand)]
#[write_component(LightStats)]
#[read_component(Position)]
#[read_component(Appearance)]
fn shadow_cast(
    world: &mut legion::world::SubWorld,
    #[state(true)] first: &mut bool,
    #[subscriber] node_additions: impl Iterator<Item = node::AddEvent>,
    #[subscriber] node_post_removals: impl Iterator<Item = node::PostRemoveEvent>,
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
        let mut query = <(&mut LightStats, &Position, &Appearance)>::query();

        struct Marker<'t> {
            light: &'t Cell<Brightness>,
            min: [Finite; 2],
            max: [Finite; 2],
            priority: Finite,
        }

        let mut markers = Vec::new();

        for (stats, &position, appearance) in query.iter_mut(world) {
            // Sun rotates from +x towards +y, normal to +z
            let yaw: f64 = PI * 2. / MONTH_COUNT.small_float::<f64>() * month.small_float::<f64>();

            // rot is the rotation matrix from the real coordinates to the time when yaw=0
            let rot = nalgebra::Rotation3::from_axis_angle(&Vector::z_axis(), -yaw)
                .matrix()
                .to_homogeneous();

            let light = stats.brightness.get_mut(month).expect("month < MONTH_COUNT");
            *light = Brightness(0.);

            let light: &Cell<Brightness> = Cell::from_mut(light);

            for component in appearance.components() {
                let trans = component.transform(position);
                let (min, max) = component.unit().bb_under(rot * trans);
                let priority = Finite::new(max.x);
                let marker = Marker {
                    light,
                    min: [Finite::new(min.y), Finite::new(min.z)],
                    max: [Finite::new(max.y), Finite::new(max.z)],
                    priority,
                };
                markers.push(marker);
            }
        }

        let cuts: SmallVec<[Vec<Finite>; 2]> = (0_usize..2)
            .map(|axis| {
                let mut vec: Vec<_> = markers
                    .iter()
                    .map(|marker| marker.min[axis])
                    .chain(markers.iter().map(|marker| marker.max[axis]))
                    .collect();
                vec.sort_unstable();
                vec
            })
            .collect();

        // If grids[(i, j)] == k, markers[k] is the highest marker in the grid
        // from (cuts[0][i], cuts[1][j]) to (cuts[0][i + 1], cuts[1][j + 1])
        let mut grids = BTreeMap::<(usize, usize), usize>::new();
        for (marker_index, marker) in markers.iter().enumerate() {
            let min_grid_index: SmallVec<[usize; 2]> = (0_usize..2)
                .map(|axis| {
                    cuts[axis].binary_search(&marker.min[axis]).expect("Cut was inserted to Vec")
                })
                .collect();
            let max_grid_index: SmallVec<[usize; 2]> = (0_usize..2)
                .map(|axis| {
                    cuts[axis].binary_search(&marker.max[axis]).expect("Cut was inserted to Vec")
                })
                .collect();

            for i in min_grid_index[0]..max_grid_index[0] {
                for j in min_grid_index[1]..max_grid_index[1] {
                    let key = (i, j);
                    match grids.entry(key) {
                        Entry::Vacant(entry) => {
                            entry.insert(marker_index);
                        }
                        Entry::Occupied(mut entry) => {
                            if markers[*entry.get()].priority < marker.priority {
                                entry.insert(marker_index);
                            }
                        }
                    }
                }
            }
        }
        log::debug!("Split objects into {} grids", grids.len());

        for ((i, j), marker_index) in grids {
            let len0 = cuts[0][i + 1].value() - cuts[0][i].value();
            let len1 = cuts[1][j + 1].value() - cuts[1][j].value();
            let area = Brightness(len0 * len1);
            let light = markers[marker_index].light;
            light.set(light.get() + area);
        }
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(move_sun_setup).uses(shadow_cast_setup)
}
