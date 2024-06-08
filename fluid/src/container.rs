use dynec::{archetype, comp, system, Entity};
use rayon::iter::ParallelIterator;
use traffloat_graph::building::Facility;
use traffloat_graph::corridor::Duct;

use crate::{Mass, Pressure, Type, TypeDefs, Volume};

archetype! {
    /// A container holds zero or more types of fluids.
    pub Container;
}

/// The facility or duct that owns this duct.
#[comp(of = Container, required)]
pub enum Owner {
    Facility(#[entity] Entity<Facility>),
    Duct(#[entity] Entity<Duct>),
}

/// An optional component indicating that the facility/duct has a fluid container.
#[comp(of = Facility, of = Duct)]
pub struct ContainerRef {
    /// Reference to the container entity.
    #[entity]
    pub storage: Entity<Container>,
}

/// The amount of a fluid of a type in a container.
#[derive(Debug, Clone, Copy)]
#[comp(of = Container, isotope = Type, required, init = || TypedMass { mass: Mass { quantity: 0.0 } })]
pub struct TypedMass {
    pub mass: Mass,
}

/// The space occupied by fluid of a type in a container.
#[derive(Debug, Clone, Copy)]
#[comp(of = Container, required, init = || CurrentVolume { volume: Volume { quantity: 0.0 } })]
pub struct CurrentVolume {
    pub volume: Volume,
}

/// The pressure force exerted by a fluid on a container.
///
/// This value is never negative.
#[derive(Debug, Clone, Copy)]
#[comp(of = Container, required, init = || CurrentPressure { pressure: Pressure { quantity: 0.0 } })]
pub struct CurrentPressure {
    pub pressure: Pressure,
}

/// The maximum space capacity for a container.
#[derive(Debug, Clone, Copy)]
#[comp(of = Container, required)]
pub struct MaxVolume {
    pub volume: Volume,
}

/// The maximum pressure of fluid in a storage, above which the container explodes.
#[derive(Debug, Clone, Copy)]
#[comp(of = Container, required)]
pub struct MaxPressure {
    pub pressure: Pressure,
}

/// The current fluid phase of a container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[comp(of = Container, required, init = || ContainerPhase::Vacuum)]
pub enum ContainerPhase {
    /// There is vacuum space in the container.
    Vacuum,
    /// The container is full, and the mixture is compressing.
    ///
    /// This is either the compression phase or the saturation phase.
    Compression,
    /// The container will explode in the next cycle
    /// if the pressure does not drop back below max pressure.
    Exploding,
}

pub const GAMMA: f64 = 65536.0;

/// Reconciles the volume and pressure of a container.
#[system]
#[allow(clippy::too_many_arguments)]
pub fn reconcile_container(
    container_iter: system::EntityIterator<Container>,
    mut current_volume_write: system::WriteSimple<Container, CurrentVolume>,
    mut current_pressure_write: system::WriteSimple<Container, CurrentPressure>,
    max_volume_read: system::ReadSimple<Container, MaxVolume>,
    max_pressure_read: system::ReadSimple<Container, MaxPressure>,
    mut phase_write: system::WriteSimple<Container, ContainerPhase>,
    #[dynec(global)] defs: &TypeDefs,
    mass_read: system::ReadIsotopeFull<Container, TypedMass>,
) {
    container_iter
        .par_entities_with_chunked((
            &mut current_volume_write,
            &mut current_pressure_write,
            &max_volume_read,
            &max_pressure_read,
            &mut phase_write,
        ))
        .for_each(
            |(entity, (current_volume, current_pressure, &max_volume, &max_pressure, phase))| {
                // TODO optimize away fluid types with negligible mass

                let last_phase = *phase;

                let total_mass: f64 =
                    mass_read.get_all(&entity).map(|(_, &TypedMass { mass })| mass.quantity).sum();

                let vacuum_volume = mass_read
                    .get_all(&entity)
                    .map(|(ty, &TypedMass { mass })| {
                        mass.quantity * defs.get(ty).vacuum_specific_volume
                    })
                    .sum::<f64>();
                if vacuum_volume < max_volume.volume.quantity {
                    *phase = ContainerPhase::Vacuum;
                    current_volume.volume = Volume { quantity: vacuum_volume };
                    current_pressure.pressure = Pressure { quantity: total_mass / vacuum_volume };
                    return;
                }

                current_volume.volume = max_volume.volume;

                let base_pressure = total_mass / max_volume.volume.quantity;

                let prev_pressure = current_pressure.pressure;
                current_pressure.pressure = Pressure {
                    quantity: mass_read
                        .get_all(&entity)
                        .map(|(ty, &TypedMass { mass })| {
                            let critical_pressure = defs.get(ty).critical_pressure.quantity;
                            if critical_pressure < base_pressure {
                                mass.quantity
                                    * (GAMMA / max_volume.volume.quantity
                                        + critical_pressure * (1.0 - GAMMA) / total_mass)
                            } else {
                                mass.quantity / max_volume.volume.quantity
                            }
                        })
                        .sum::<f64>(),
                };

                if prev_pressure > max_pressure.pressure
                    && current_pressure.pressure > max_pressure.pressure
                {
                    *phase = ContainerPhase::Exploding;
                    if last_phase == ContainerPhase::Exploding {
                        // TODO send ExplosionEvent
                    }
                }
            },
        )
}

#[cfg(test)]
mod tests;
