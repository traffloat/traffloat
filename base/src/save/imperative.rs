//! An imperative is an entry type in a save file.

use bevy::ecs::bundle;
use bevy::ecs::component::Component;

/// Components to construct an imperative entity.
#[derive(bundle::Bundle)]
pub struct Bundle {
    pub key: Key,
}

/// A string consistent across versions,
/// used to identify the imperative type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Component)]
pub struct Key(pub &'static str);

/// Indicates the imperative types that must be loaded before this type.
#[derive(Component)]
pub struct Dependencies {
    pub keys: Vec<Key>,
}
