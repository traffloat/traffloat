use bevy::ecs::schedule::SystemSet;

pub mod proto;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct UpdateHandlerSystemSet;
