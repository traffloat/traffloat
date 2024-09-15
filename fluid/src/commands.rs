use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, Query, SystemState};
use bevy::ecs::world::{Command, World};
use bevy::hierarchy::{self, BuildChildren};
use traffloat_graph::corridor::Binary;
use typed_builder::TypedBuilder;

use crate::{config, container, pipe, units};

/// A command to create a new container element.
#[derive(TypedBuilder)]
pub struct CreateContainerElement {
    /// The container to add element to.
    pub container: Entity,
    /// The element fluid type.
    pub ty:        config::Type,
    /// The initial mass of fluid.
    #[builder(setter(into))]
    pub mass:      units::Mass,
}

impl Command for CreateContainerElement {
    fn apply(self, world: &mut World) {
        fn populate_pipe(
            commands: &mut Commands,
            pipe_query: &Query<(Option<&hierarchy::Children>, &pipe::Containers), ()>,
            pipe_element_query: &mut Query<(&config::Type, &mut pipe::element::ContainerElements)>,
            container_entity: Entity,
            target_ty: config::Type,
            pipe_entity: Entity,
            container_element_entity: Entity,
        ) {
            let (pipe_elements, container_endpoints) =
                pipe_query.get(pipe_entity).expect("Pipes must contain pipe entities");
            let target_endpoint_in_pipe =
                container_endpoints.endpoints.find(&container_entity).expect(
                    "each pipe in container adjacency list must have one endpoint as the container",
                );

            for &pipe_element in pipe_elements.into_iter().flatten() {
                let Ok((&element_ty, mut container_element_endpoints)) =
                    pipe_element_query.get_mut(pipe_element)
                else {
                    continue;
                };
                if element_ty == target_ty {
                    *container_element_endpoints
                        .containers
                        .as_endpoint_mut(target_endpoint_in_pipe) = Some(container_element_entity);
                    return;
                }
            }

            commands.entity(pipe_entity).with_children(|builder| {
                builder.spawn(
                    pipe::element::Bundle::builder()
                        .ty(target_ty)
                        .container_elements(pipe::element::ContainerElements {
                            containers: {
                                let mut container_elements = Binary::from_fn(|_| None);
                                *container_elements.as_endpoint_mut(target_endpoint_in_pipe) =
                                    Some(container_element_entity);
                                container_elements
                            },
                        })
                        .build(),
                );
            });
        }

        let container_element =
            world.spawn(container::element::Bundle::builder().ty(self.ty).mass(self.mass).build());
        let container_element = container_element.id();

        let mut state = SystemState::<(
            Commands,
            Query<&container::Pipes>,
            Query<(Option<&hierarchy::Children>, &pipe::Containers)>,
            Query<(&config::Type, &mut pipe::element::ContainerElements)>,
        )>::new(world);
        let (mut commands, container_query, pipe_query, mut pipe_element_query) =
            state.get_mut(world);

        let pipes = container_query
            .get(self.container)
            .expect("CreateContainerElement.container must be a container entity");

        for &pipe in &pipes.pipes {
            populate_pipe(
                &mut commands,
                &pipe_query,
                &mut pipe_element_query,
                self.container,
                self.ty,
                pipe,
                container_element,
            );
        }
    }
}
