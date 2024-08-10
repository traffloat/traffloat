//! Manages the viewers of a Traffloat world.
//!
//! This crate is used for both single-player and multi-player servers.
//! It provides an abstract layer of communication
//! between the simulation modules ("server") and the visualization modules ("client").

use std::hash::Hash;
use std::sync::atomic::{self, AtomicU32};

use bevy::app::{self, App};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::Resource;
use bevy::ecs::world::World;
use bevy::utils::HashMap;

pub mod metrics;
pub mod viewable;
pub mod viewer;

/// Initializes the view framework.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((viewable::Plugin, viewer::Plugin, metrics::Plugin));
    }
}

/// Maps serialization-level IDs to entities.
#[derive(Resource)]
pub struct IdIndex<Id> {
    map:     HashMap<Id, Entity>,
    next_id: AtomicU32,
}

impl<Id> IdIndex<Id>
where
    Id: From<u32> + Copy + Eq + Hash + Component,
{
    /// Resolves an ID from clients into the actual entity.
    pub fn get(&self, id: Id) -> Option<Entity> { self.map.get(&id).copied() }

    /// Initializes this index for a world.
    pub fn init(world: &mut World) {
        world.insert_resource(Self { map: HashMap::new(), next_id: AtomicU32::new(0) });

        let hooks = world.register_component_hooks::<Id>();
        hooks.on_add(|mut world, entity, _comp_id| {
            let &id = world.get::<Id>(entity).expect("subject of component hook");
            world.resource_mut::<Self>().map.insert(id, entity);
        });
        hooks.on_remove(|mut world, entity, _comp_id| {
            let &id = world.get::<Id>(entity).expect("subject of component hook");
            let removed = world.resource_mut::<Self>().map.remove(&id);
            assert_eq!(removed, Some(entity), "indexed ID does not match actual entity");
        });
    }

    /// Request a new ID in a system without exclusively locking `IdIndex`.
    pub fn next_id(&self) -> Id { Id::from(self.next_id.fetch_add(1, atomic::Ordering::SeqCst)) }

    /// Request a new ID.
    pub fn next_id_mut(&mut self) -> Id {
        let next_id = self.next_id.get_mut();
        let id = *next_id;
        *next_id += 1;

        Id::from(id)
    }
}
