//! A container is an object in which fluid is stored.
//!
//! Both storages and pipes are considered containers.
//!
//! Each container is the parent entity of a number of "container elements" child entities,
//! corresponding to all present fluid types in the container.
//!
//! A storage for a [facility] should share the same entity as the facility entity.
//! A container for a [duct] should share the same entity as the duct entity.

use std::iter;

use bevy::app::{self, App};
use bevy::ecs::bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::schedule::{IntoSystemConfigs, SystemSet};
use bevy::ecs::system::{Commands, Query};
use bevy::ecs::world::World;
use bevy::hierarchy;
use bevy::state::condition::in_state;
use bevy::state::state::States;
use derive_more::From;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use traffloat_base::save;
use traffloat_graph::building::facility;
use traffloat_graph::corridor::duct;
use typed_builder::TypedBuilder;

use crate::{config, units};

pub mod element;

mod metrics;

#[cfg(test)]
mod tests;

/// Maintains the state within each container.
pub(crate) struct Plugin<St>(pub(super) St);

impl<St: States + Copy> app::Plugin for Plugin<St> {
    fn build(&self, app: &mut App) {
        app.add_plugins(metrics::Plugin(self.0));

        app.add_systems(
            app::Update,
            rebalance_system.in_set(SystemSets::Rebalance).run_if(in_state(self.0)),
        );
        save::add_def::<Save>(app);
        save::add_def::<element::Save>(app);
    }
}

/// System sets for container fluids.
#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum SystemSets {
    /// Rebalance volume and pressure within each container based on the mass.
    ///
    /// [`element::Mass`]-mutating systems should execute before this set.
    /// Systems that read [`CurrentVolume`], [`CurrentPressure`], [`element::Volume`] or
    /// [`ExplosionMarker`] should execute after this set.
    Rebalance,
}

/// Components to construct a container.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    #[builder(default = CurrentPressure { pressure: <_>::default() })]
    current_pressure: CurrentPressure,
    #[builder(default = CurrentVolume { volume: <_>::default() })]
    current_volume:   CurrentVolume,
    #[builder(setter(into))]
    max_volume:       MaxVolume,
    #[builder(setter(into))]
    max_pressure:     MaxPressure,
    #[builder(default = Pipes { pipes: <_>::default() })]
    pipes:            Pipes,
    #[builder(default, setter(skip))]
    _marker:          Marker,
}

/// Marks an entity as a container.
#[derive(Component, Default)]
pub struct Marker;

/// Overall pressure of a container.
#[derive(Component)]
pub struct CurrentPressure {
    /// Current pressure value.
    pub pressure: units::Pressure,
}

/// Total volume occupied by fluids in a container.
///
/// `MaxVolume - OccupiedVolume` is contains vacuum.
#[derive(Component)]
pub struct CurrentVolume {
    /// Current volume value.
    pub volume: units::Volume,
}

/// Volume capacity available in a container.
///
/// The occupied volume never (significantly) exceeds this value.
#[derive(Component, From)]
pub struct MaxVolume {
    /// Max volume value.
    pub volume: units::Volume,
}

/// The explosion threshold of a container.
///
/// A container entity explodes (with the [`ExplosionMarker`] set)
/// if the pressure exceeds the threshold for two consecutive cycles
/// and the pressure of the latter cycle is greater than the previous one.
#[derive(Component, From)]
pub struct MaxPressure {
    /// Max pressure value.
    pub pressure: units::Pressure,
}

/// List of adjacent pipes of a container.
#[derive(Component)]
pub struct Pipes {
    /// List of pipe entities.
    pub pipes: SmallVec<[Entity; 3]>,
}

/// A marker component on containers indicating that it has exploded.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct ExplosionMarker;

/// Rebalance the volume of fluids in a system.
fn rebalance_system(
    types: config::Types,
    mut containers_query: Query<(
        Entity,
        &hierarchy::Children,
        &mut CurrentPressure,
        &mut CurrentVolume,
        &MaxVolume,
        &MaxPressure,
    )>,
    mut elements_query: Query<(&config::Type, &element::Mass, &mut element::Volume)>,
    mut commands: Commands,
) {
    #[derive(Default)]
    struct ElementState {
        critical_pressure: units::Pressure,
        saturation_gamma:  f32,
    }

    let mut buf = Vec::<Option<ElementState>>::default();

    containers_query.iter_mut().for_each(
        |(container_entity, elements, mut pressure, mut occupied, max_volume, max_pressure)| {
            buf.resize_with(elements.len(), <_>::default);

            let previous_pressure = pressure.pressure;
            let mut total_vacuum_volume = units::Volume { quantity: 0. };

            // First compute the vacuum volume and temporarily save them in the current volume component.
            // Even if they won't end up as the eventual value if it is not vacuum phase,
            // this would serve as a buffer memory.
            for (state, &element) in iter::zip(&mut buf, elements) {
                let Ok((&ty, mass, mut volume)) = elements_query.get_mut(element) else { continue };
                let def = types.get(ty);

                *state = Some(ElementState {
                    critical_pressure: def.critical_pressure,
                    saturation_gamma:  def.saturation_gamma,
                });

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
                let Some(state) = state else { continue };

                // scale volume proportionally to add up to approximately max_volume
                let (_, _, mut volume) = elements_query
                    .get_mut(element)
                    .expect("state.is_some() iff child is an element");
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

/// Save schema.
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Save {
    /// Reference to parent facility or duct.
    pub owner:        SaveOwner,
    /// Container capacity.
    pub max_volume:   units::Volume,
    /// Container pressure limit.
    pub max_pressure: units::Pressure,
}

/// Owner of the container, used in saves.
#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum SaveOwner {
    /// Container is a facility storage.
    Facility(save::Id<facility::Save>),
    /// Container is a duct buffer.
    Duct(save::Id<duct::Save>),
}

impl save::Def for Save {
    const TYPE: &'static str = "traffloat.save.fluid.Container";

    type Runtime = Entity;

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(
            mut writer: save::Writer<Save>,
            (facility_dep, duct_dep): (
                save::StoreDepend<facility::Save>,
                save::StoreDepend<duct::Save>,
            ),
            (query, owner_marker_query): (
                Query<(Entity, &MaxVolume, &MaxPressure), With<Marker>>,
                Query<(Option<&facility::Marker>, Option<&duct::Marker>)>,
            ),
        ) {
            writer.write_all(query.iter().map(|(entity, max_volume, max_pressure)| {
                let save_parent = match owner_marker_query
                    .get(entity)
                    .expect("dangling parent reference")
                {
                    (Some(_), Some(_)) => unreachable!("entity cannot be both facility and duct"),
                    (Some(_), None) => SaveOwner::Facility(facility_dep.must_get(entity)),
                    (None, Some(_)) => SaveOwner::Duct(duct_dep.must_get(entity)),
                    (None, None) => {
                        panic!("container must be the same entity as a facility or a duct")
                    }
                };

                (
                    entity,
                    Save {
                        owner:        save_parent,
                        max_volume:   max_volume.volume,
                        max_pressure: max_pressure.pressure,
                    },
                )
            }));
        }

        save::StoreSystemFn::new(store_system)
    }

    fn loader() -> impl save::LoadOnce<Def = Self> {
        #[allow(clippy::trivially_copy_pass_by_ref, clippy::unnecessary_wraps)]
        fn loader(
            world: &mut World,
            def: Save,
            (facility_dep, duct_dep): &(
                save::LoadDepend<facility::Save>,
                save::LoadDepend<duct::Save>,
            ),
        ) -> anyhow::Result<Entity> {
            let owner = match def.owner {
                SaveOwner::Facility(owner) => facility_dep.get(owner)?,
                SaveOwner::Duct(owner) => duct_dep.get(owner)?,
            };
            let bundle = Bundle::builder()
                .max_volume(def.max_volume)
                .max_pressure(def.max_pressure)
                .pipes(Pipes { pipes: <_>::default() })
                .build();

            let mut owner_entity = world.entity_mut(owner);
            let container = owner_entity.insert(bundle);
            Ok(container.id())
        }

        save::LoadFn::new(loader)
    }
}
