use std::hash::Hash;

use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, Resource};
use bevy::hierarchy::{self, BuildChildren};
use bevy::utils::HashMap;

/// Index of delegate entities synchronized from the server.
#[derive(Resource)]
pub struct SidIndex<Sid> {
    map: HashMap<Sid, Entity>,
}

impl<Sid> Default for SidIndex<Sid> {
    fn default() -> Self { Self { map: HashMap::default() } }
}

impl<Sid: Copy + Eq + Hash + Send + Sync + 'static> SidIndex<Sid> {
    pub fn add<BundleT: Bundle, ExtraBundleT: Bundle>(
        &mut self,
        sid: Sid,
        commands: &mut Commands,
        spawn_entity: impl FnOnce() -> BundleT,
        spawn_children: impl FnOnce(&mut hierarchy::ChildBuilder) -> ExtraBundleT,
    ) -> Entity {
        *self.map.entry(sid).or_insert_with(move || {
            let mut delegate_entity = commands.spawn((spawn_entity(), Marker(sid)));

            // who wants to chain with_children anyway...
            let mut extra = None;
            delegate_entity.with_children(|b| {
                extra = Some(spawn_children(b));
            });

            delegate_entity.insert(extra.expect("spawn_children must be called"));

            delegate_entity.id()
        })
    }

    pub fn get(&self, sid: Sid) -> Option<Entity> { self.map.get(&sid).copied() }
}

/// Marks that an entity is the delegate of the specified SID.
#[derive(Component)]
pub struct Marker<Sid>(pub Sid);
