//! A container is an object in which fluid is stored.
//!
//! Both storages and pipes are considered containers.
//!
//! Each container is the parent entity of a number of "container elements" child entities,
//! corresponding to all present fluid types in the container.

use std::iter;

use bevy::ecs::bundle;
use bevy::prelude::{Commands, Component, Entity, Query, Res};
use bevy::{app, hierarchy};
use typed_builder::TypedBuilder;

use crate::config::{self, TypeDefs};
use crate::units;

pub mod element;

#[cfg(test)]
mod tests;

/// Maintains the state within each container.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(app::Update, rebalance_system);
    }
}

/// Components to construct a container.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    #[builder(default = CurrentPressure { pressure: <_>::default() })]
    current_pressure: CurrentPressure,
    #[builder(default = CurrentVolume { volume: <_>::default() })]
    current_volume:   CurrentVolume,
    max_volume:       MaxVolume,
    max_pressure:     MaxPressure,
}

/// Overall pressure of a container.
#[derive(Component)]
pub struct CurrentPressure {
    pub pressure: units::Pressure,
}

/// Total volume occupied by fluids in a container.
///
/// `MaxVolume - OccupiedVolume` is contains vacuum.
#[derive(Component)]
pub struct CurrentVolume {
    pub volume: units::Volume,
}

/// Volume capacity available in a container.
///
/// The occupied volume never (significantly) exceeds this value.
#[derive(Component)]
pub struct MaxVolume {
    pub volume: units::Volume,
}

/// The explosion threshold of a container.
///
/// A container entity explodes (with the [`ExplosionMarker`] set)
/// if the pressure exceeds the threshold for two consecutive cycles
/// and the pressure of the latter cycle is greater than the previous one.
#[derive(Component)]
pub struct MaxPressure {
    pub pressure: units::Pressure,
}

/// A marker component on containers indicating that it has exploded.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct ExplosionMarker;

/// Rebalance the volume of fluids in a system.
fn rebalance_system(
    defs: Res<TypeDefs>,
    mut q_containers: Query<(
        Entity,
        &hierarchy::Children,
        &mut CurrentPressure,
        &mut CurrentVolume,
        &MaxVolume,
        &MaxPressure,
    )>,
    mut q_elements: Query<(&config::Type, &element::Mass, &mut element::Volume)>,
    mut commands: Commands,
) {
    #[derive(Default)]
    struct ElementState {
        critical_pressure: units::Pressure,
        saturation_gamma:  f32,
    }

    let mut buf = Vec::<ElementState>::default();

    q_containers.iter_mut().for_each(
        |(container_entity, elements, mut pressure, mut occupied, max_volume, max_pressure)| {
            buf.resize_with(elements.len(), <_>::default);

            let previous_pressure = pressure.pressure;
            let mut total_vacuum_volume = units::Volume { quantity: 0. };

            // First compute the vacuum volume and temporarily save them in the current volume component.
            // Even if they won't end up as the eventual value if it is not vacuum phase,
            // this would serve as a buffer memory.
            for (state, &element) in iter::zip(&mut buf, elements) {
                let (&ty, mass, mut volume) = q_elements.get_mut(element).unwrap();
                let def = defs.get(ty);

                *state = ElementState {
                    critical_pressure: def.critical_pressure,
                    saturation_gamma:  def.saturation_gamma,
                };

                volume.volume = mass.mass * def.vacuum_specific_volume;
                total_vacuum_volume += volume.volume;
            }

            let base_pressure = units::Pressure {
                quantity: total_vacuum_volume.quantity / max_volume.volume.quantity,
            };
            pressure.pressure = base_pressure;

            // vacuum phase
            if base_pressure.quantity <= 1. {
                occupied.volume = total_vacuum_volume;
                return;
            }

            occupied.volume = max_volume.volume;

            let mut saturated_pressure = base_pressure;
            for (state, &element) in iter::zip(&buf, elements) {
                // scale volume proportionally to add up to approximately max_volume
                let (_, _, mut volume) = q_elements.get_mut(element).unwrap();
                volume.volume.quantity /= base_pressure.quantity;

                if base_pressure > state.critical_pressure {
                    let additional = (base_pressure - state.critical_pressure).quantity
                        * volume.volume.quantity
                        / max_volume.volume.quantity;
                    saturated_pressure.quantity += additional * state.saturation_gamma;
                }
            }

            pressure.pressure = saturated_pressure;

            if saturated_pressure > previous_pressure && previous_pressure > max_pressure.pressure {
                commands.entity(container_entity).insert(ExplosionMarker);
            }
        },
    );
}
