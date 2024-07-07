//! Force is a directed modifier on the flow rate of a pipe.
//!
//! There is a force pushing fluids in each container to the other container in each pipe.
//! The force is the sum of different directed sources of force
//! provided by [additive modifiers](SystemSets::Additive),
//! such as pressure difference, pumps and fields.
//!
//! After ensuring the sum is non-negative (by clamping to zero if negative),
//! [relative modifiers](SystemSets::Relative) apply directed proportional changes to the forces.
//! Note that if the proportional change is identical for both directions,
//! it should be provided as a [resistance](../resistance) source instead
//! to reduce half of the operations.
//!
//! All modifiers operate on the [`Directed`] component.

use bevy::app;
use bevy::prelude::{App, Component, IntoSystemConfigs, IntoSystemSetConfigs, Query, SystemSet};
use traffloat_graph::corridor::Binary;

use super::{resistance, Containers};
use crate::{container, units};

pub(super) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            app::Update,
            (
                init_force.before(SystemSets::Additive).in_set(SystemSets::Compute),
                apply_resistance
                    .after(SystemSets::Additive)
                    .before(SystemSets::Relative)
                    .after(resistance::SystemSets::Compute),
            ),
        );
        app.configure_sets(
            app::Update,
            (SystemSets::Additive, SystemSets::Relative).in_set(SystemSets::Compute),
        );
    }
}

const VOLUME_PER_PRESSURE_DELTA: f32 = 1.;

fn init_force(
    mut pipe_query: Query<(&mut Directed, &Containers)>,
    container_query: Query<&container::CurrentPressure>,
) {
    for (mut directed, containers) in pipe_query.iter_mut() {
        let pressure = containers.endpoints.query(&container_query).map(|comp| comp.pressure);
        let ab = (pressure.alpha - pressure.beta).quantity * VOLUME_PER_PRESSURE_DELTA;
        directed.force.alpha = units::Volume { quantity: ab };
        directed.force.beta = units::Volume { quantity: -ab };
    }
}

fn apply_resistance(mut query: Query<(&mut Directed, &resistance::Dynamic)>) {
    for (mut directed, resistance) in query.iter_mut() {
        directed.force.each_mut(|force| {
            force.quantity = force.quantity.max(0.);
            force.quantity /= resistance.resistance;
        })
    }
}

/// The force acting on each side of the pipe.
#[derive(Component)]
pub struct Directed {
    /// The resultant value is the gross directed volumetric flow of liquid,
    /// so the type also uses the `Volume` unit.
    pub force: Binary<units::Volume>,
}

/// System sets for resistance processing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum SystemSets {
    /// A system set wrapping all systems for initializing and computing force.
    Compute,
    /// Systems that modify forces by adding/subtracting values.
    Additive,
    /// Systems that modify forces by a multiplier.
    Relative,
}
