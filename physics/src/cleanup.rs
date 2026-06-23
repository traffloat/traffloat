use std::mem;

use bevy::app::App;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Commands, Query, SystemState};
use bevy::ecs::world::World;

/// Marker component for a root entity in the physics simulation.
///
/// All physics entities that do not have a `linked_spawn` relationship must have this component
/// to facilitate proper teardown when the physics world is unloaded.
#[derive(Component)]
pub struct WorldObject;

impl WorldObject {
    pub(super) fn cleanup_hook(world: &mut World) {
        let mut state = SystemState::<(Commands, Query<Entity, With<WorldObject>>)>::new(world);
        let (mut commands, query) = state.get_mut(world);
        for entity in query {
            commands.entity(entity).despawn();
        }
        state.apply(world);
    }
}

#[derive(Resource, Default)]
pub struct CleanupHooks {
    pub hooks: Vec<Box<dyn Fn(&mut World) + Send + Sync>>,
}

pub trait CleanupAppExt {
    fn add_cleanup_hook(&mut self, hook: impl Fn(&mut World) + Send + Sync + 'static);
}

impl CleanupAppExt for App {
    /// Registers a cleanup hook to be called when the physics world is unloaded/reloaded.
    ///
    /// The [`CleanupHooks`] resource is swapped out during cleanup hook call to avoid aliasing,
    /// so hooks must not interact with the `CleanupHooks` resource during execution.
    fn add_cleanup_hook(&mut self, hook: impl Fn(&mut World) + Send + Sync + 'static) {
        self.world_mut().resource_mut::<CleanupHooks>().hooks.push(Box::new(hook));
    }
}

pub fn execute_cleanup_hooks(world: &mut World) {
    let hooks = mem::take(&mut world.resource_mut::<CleanupHooks>().hooks);
    for hook in &hooks {
        hook(world);
    }
    world.resource_mut::<CleanupHooks>().hooks = hooks;
}
