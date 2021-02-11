//! Calculates the sunlight level of each building

use std::cell::RefCell;
use std::cmp;
use std::collections::BTreeSet;
use std::f64::consts::PI;

use legion::Entity;

use crate::graph::*;
use crate::shape::Shape;
use crate::types::{Clock, Position, ScalarConfig, Vector};
use crate::SetupEcs;

/// The position of the sun
#[derive(Default, getset::CopyGetters)]
pub struct Sun {
    /// Orientation of the sun, in radians from +x towards +y
    #[getset(get_copy = "pub")]
    yaw: f64,
}

impl Sun {
    /// Direction vector
    pub fn direction(&self) -> Vector {
        Vector::new(self.yaw().cos(), self.yaw().sin(), 0.)
    }
}

#[codegen::system]
fn move_sun(
    #[resource] sun: &mut Sun,
    #[resource] clock: &Clock,
    #[resource] config: &ScalarConfig,
) {
    sun.yaw += config.sun_speed * clock.delta;
    sun.yaw %= PI * 2.;
}

/// Number of partitions to compute shadow casting for
pub const MONTH_COUNT: usize = 12;

/// A component storing the lighting data for a node.
#[derive(Debug, Default)]
pub struct LightStats {
    /// The brightness values in each partition.
    ///
    /// The brightness value is the length receiving sunlight.
    pub brightness: [f64; MONTH_COUNT],
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

    #[derive(Debug)]
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
        todo!("rewrite")
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup
        .resource(Sun::default())
        .uses(move_sun_setup)
        .uses(shadow_cast_setup)
}
