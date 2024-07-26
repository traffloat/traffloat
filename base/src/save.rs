use std::collections::HashMap;

use bevy::ecs::component::Component;
use bevy::ecs::world::World;

pub struct Types {
    types: HashMap<Type, TypeDef>,
}

/// A string consistent across versions,
/// used to identify the imperative type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Component)]
pub struct Type(pub &'static str);

pub struct TypeDef {
    pub dependencies: Vec<Type>,
    pub execute:      Box<SaveExecutor>,
}

type SaveExecutor = dyn Fn(&mut World, &mut Vec<prost_types::Any>);
