use bevy::app::{self, App, Plugin};
use bevy::ecs::world::World;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::types;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<TypeDef>();

        types::init::<TypeDef>(app);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct TypeDef {
    pub name:           String,
    pub granule_mass:   f32,
    pub granule_volume: f32,
}

types::define_type! {
    "cargo", "cargo:type", TypeDef;
    TypeId, PersistDeps, Types, PersistTypes, TypesGeneration;
    depends {}
}
