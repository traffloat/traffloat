//! A pipe is a link between two containers.
//!
//! Despite a possible implication of its name,
//! pipes exist within a building or between a building and a corridor,
//! but not in the corridor.
//!
//! Each pipe is the parent entity of a number of "pipe element" child entities,
//! corresponding to all active fluid types across the link.
//!
//! In each simulation cycle, the following sequence of events takes place:
//! 1. Compute the [resistance] of each pipe.
//! 2. Add the [force] in each direction to the resistance
//!    as the [directed gross flow](force::Directed).
//! 3. Compute the [base transfer weight](element::TransferWeight) of each pipe element.
//! 4. Distribute the available flow rate for each directed pipe element.
//! 5. Perform container element mass updates, lazily creating/deleting pipe elements during the process.
//!
//! A storage for a intra-building inter-facility connections
//! should reference the building entity as its parent.
//! A storage for a the connection from a facility to a duct
//! should reference the duct entity as its parent.

use bevy::app::App;
use bevy::ecs::bundle;
use bevy::ecs::component::{Component, ComponentId};
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Query, Res};
use bevy::ecs::world::{DeferredWorld, World};
use bevy::hierarchy::{BuildWorldChildren, DespawnRecursiveExt};
use bevy::state::condition::in_state;
use bevy::state::state::States;
use bevy::{app, hierarchy};
use derive_more::From;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use traffloat_base::save;
use traffloat_graph::building::facility;
use traffloat_graph::corridor::{duct, Binary};
use typed_builder::TypedBuilder;

use crate::config::{self, Scalar};
use crate::{commands, container, units};

pub mod element;
pub mod force;
pub mod resistance;

#[cfg(test)]
mod tests;

/// Executes fluid mass transfer between containers.
pub(super) struct Plugin<St>(pub(super) St);

impl<St: States + Copy> app::Plugin for Plugin<St> {
    fn build(&self, app: &mut App) {
        app.add_plugins((resistance::Plugin(self.0), force::Plugin(self.0)));
        app.add_systems(
            app::Update,
            (
                update_transfer_weight_system,
                distribute_transfer_weight_system
                    .after(update_transfer_weight_system)
                    .after(force::SystemSets::Compute)
                    .before(container::SystemSets::Rebalance),
            )
                .run_if(in_state(self.0)),
        );

        app.world_mut()
            .register_component_hooks::<container::element::Mass>()
            .on_remove(remove_element_hook);
    }
}

/// Components to construct a pipe entity.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    #[builder(setter(into))]
    containers:         Containers,
    #[builder(setter(into))]
    shape_resistance:   resistance::FromShape,
    #[builder(default = resistance::Static { resistance: <_>::default() })]
    static_resistance:  resistance::Static,
    #[builder(default = resistance::Dynamic { resistance: <_>::default() })]
    dynamic_resistance: resistance::Dynamic,
    #[builder(default = force::Directed { force: <_>::default() })]
    force:              force::Directed,
    #[builder(default, setter(skip))]
    _marker:            Marker,
}

/// Marks an entity as a pipe.
#[derive(Component, Default)]
pub struct Marker;

/// The containers connected by the pipe.
#[derive(Component, From)]
pub struct Containers {
    /// Endpoint container references.
    pub endpoints: Binary<Entity>,
}

fn update_transfer_weight_system(
    types: config::Types,
    mut pipe_elements_query: Query<(
        &mut element::TransferWeight,
        &config::Type,
        &element::ContainerElements,
    )>,
    container_elements_query: Query<(&container::element::Volume, &hierarchy::Parent)>,
    containers_query: Query<&container::CurrentVolume>,
) {
    pipe_elements_query.iter_mut().for_each(|(mut weights_write, &ty, endpoints)| {
        let def = types.get(ty);

        weights_write.output = endpoints.containers.as_ref().map(|&entity| {
            let concentration = entity.map_or(0., |entity| {
                let (volume, parent) = container_elements_query
                    .get(entity)
                    .expect("ContainerElements must contain a valid container element entity");
                let total_volume = containers_query
                    .get(parent.get())
                    .expect("Parent of container element must be a container entity")
                    .volume;
                volume.volume.quantity / total_volume.quantity
            });
            concentration / def.viscosity.quantity
        });
    });
}

fn distribute_transfer_weight_system(
    config: Res<Scalar>,
    pipes_query: Query<(&hierarchy::Children, &force::Directed, &Containers)>,
    mut pipe_elements_query: Query<(
        &config::Type,
        &element::TransferWeight,
        &element::ContainerElements,
        &mut element::AbTransferMass,
    )>,
    mut container_elements_query: Query<(
        &mut container::element::Mass,
        &container::element::Volume,
    )>,
    mut commands: Commands,
) {
    for (elements, force, containers) in pipes_query.iter() {
        let weight_sum = elements
            .iter()
            .filter_map(|&element| pipe_elements_query.get(element).ok())
            .map(|(_, weight, _, _)| weight.output)
            .fold(Binary::<f32> { alpha: 0., beta: 0. }, |sum, element| {
                sum.zip(element).map(|(a, b)| a + b)
            });

        let volume_per_weight = force.force.zip(weight_sum).map(|(a, b)| a / b);

        for &element in elements {
            let Ok((ty, weight, container_elements, mut mass_ab)) =
                pipe_elements_query.get_mut(element)
            else {
                continue;
            };

            let volume_output = volume_per_weight.zip(weight.output).map(|(a, b)| a * b);

            let mut mass_volume_comps =
                container_elements.containers.query_mut_with_entity(&mut container_elements_query);
            let mass_output =
                mass_volume_comps.as_mut().zip(volume_output).map(|(mass_volume, volume_out)| {
                    match mass_volume {
                        Some((_, (mass, volume))) => {
                            if volume.volume.quantity > 0. {
                                mass.mass * volume_out.quantity.min(volume.volume.quantity)
                                    / volume.volume.quantity
                            } else {
                                units::Mass { quantity: 0. }
                            }
                        }
                        None => units::Mass { quantity: 0. },
                    }
                });
            mass_ab.mass = mass_output.alpha - mass_output.beta;

            mass_volume_comps
                .zip((-mass_ab.mass, mass_ab.mass))
                .zip(containers.endpoints)
                .zip(container_elements.containers)
                .each_mut(|(((mass_volume, delta_mass), container), container_element_ref)| {
                    match mass_volume {
                        None if *delta_mass < config.creation_threshold => {} // negligible mass
                        None => {
                            commands.add(
                                commands::CreateContainerElement::builder()
                                    .container(*container)
                                    .ty(*ty)
                                    .mass(*delta_mass)
                                    .build(),
                            );
                        }
                        Some((container_element, (mass_comp, _))) => {
                            mass_comp.mass += *delta_mass;
                            if mass_comp.mass < config.deletion_threshold {
                                commands.entity(*container_element).despawn_recursive();
                                *container_element_ref = None;
                            }
                        }
                    }
                });
        }
    }
}

fn remove_element_hook(mut world: DeferredWorld, container_element: Entity, _: ComponentId) {
    let ty = world
        .get::<config::Type>(container_element)
        .expect("container element must have type component");
    let container = world
        .get::<hierarchy::Parent>(container_element)
        .expect("container element must have container as parent");

    let pipes = world
        .get::<container::Pipes>(container.get())
        .expect("container element parent must be container");

    let mut commands = Vec::new(); // TODO optimize this when bevy exposes DeferredWorld splitting capability

    for &pipe in &pipes.pipes {
        for &pipe_element in world.get::<hierarchy::Children>(pipe).into_iter().flatten() {
            if world.get::<config::Type>(pipe_element) == Some(ty) {
                let endpoints = world
                    .get::<element::ContainerElements>(pipe_element)
                    .expect("pipe element must have ContainerElements component");
                if let Some(endpoint) = endpoints.containers.find(&Some(container_element)) {
                    commands.push((pipe_element, endpoint));
                }
            }
        }
    }

    for (pipe_element, endpoint) in commands {
        let mut endpoints =
            world.get_mut::<element::ContainerElements>(pipe_element).expect("checked above");

        *endpoints.containers.as_endpoint_mut(endpoint) = None;
        if endpoints.containers.iter().all(Option::is_none) {
            world.commands().entity(pipe_element).despawn_recursive();
        }
    }
}

/// Save schema.
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Save {
    /// Containers connected by this pipe.
    ///
    /// The containers must be both facility storages in the same building
    /// or one facility storage and one adjacent pipe buffer.
    pub containers:       Binary<save::Id<container::Save>>,
    /// Resistance contributed by the pipe shape.
    pub shape_resistance: units::Resistance,
}

impl save::Def for Save {
    const TYPE: &'static str = "traffloat.save.fluid.Container";

    type Runtime = Entity;

    fn store_system() -> impl save::StoreSystem<Def = Self> {
        fn store_system(
            mut writer: save::Writer<Save>,
            (container_dep,): (save::StoreDepend<container::Save>,),
            query: Query<(Entity, &Containers, &resistance::FromShape), With<Marker>>,
        ) {
            writer.write_all(query.iter().map(|(entity, containers, shape_resistance)| {
                (
                    entity,
                    Save {
                        containers:       containers
                            .endpoints
                            .map(|endpoint| container_dep.must_get(endpoint)),
                        shape_resistance: shape_resistance.resistance,
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
            (container_dep,): &(save::LoadDepend<container::Save>,),
        ) -> anyhow::Result<Entity> {
            enum Parent {
                Duct(Entity),
                Building(Entity),
            }

            let container_entities =
                def.containers.try_map(|container| container_dep.get(container))?;
            let bundle = Bundle::builder()
                .containers(Containers { endpoints: container_entities })
                .shape_resistance(def.shape_resistance)
                .build();

            let parent_candidates = container_entities.try_map(|container| {
                let container_parent = world
                    .get::<hierarchy::Parent>(container)
                    .expect("container from stored data must have parent")
                    .get();
                if world.get::<duct::Marker>(container_parent).is_some() {
                    Ok(Parent::Duct(container_parent))
                } else if world.get::<facility::Marker>(container_parent).is_some() {
                    let facility_parent = world
                        .get::<hierarchy::Parent>(container_parent)
                        .expect("facility must have a parent")
                        .get();
                    Ok(Parent::Building(facility_parent))
                } else {
                    anyhow::bail!("endpoint container parent must be a facility or a duct")
                }
            })?;
            let parent = match parent_candidates {
                Binary {
                    alpha: Parent::Building(alpha_building),
                    beta: Parent::Building(beta_building),
                } => {
                    anyhow::ensure!(
                        alpha_building == beta_building,
                        "pipe endpoint containers belong to facilities in different buildings"
                    );
                    alpha_building
                }
                Binary { alpha: Parent::Building(_), beta: Parent::Duct(duct) }
                | Binary { alpha: Parent::Duct(duct), beta: Parent::Building(_) } => duct,
                Binary { alpha: Parent::Duct(_), beta: Parent::Duct(_) } => {
                    anyhow::bail!("pipe cannot have both endpoints as duct containers");
                }
            };

            let mut container = world.spawn(bundle);
            container.set_parent(parent);
            Ok(container.id())
        }

        save::LoadFn::new(loader)
    }
}
