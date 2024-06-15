//! A container element corresponds to an active fluid type in a container.
//!
//! A container element is created when an adjacent pipe wants to transfer
//! a new fluid type into this container.

use bevy::ecs::bundle;
use bevy::prelude::Component;
use typed_builder::TypedBuilder;

use crate::{config, units};

/// Components to construct a container element.
#[derive(bundle::Bundle, TypedBuilder)]
pub struct Bundle {
    ty:     config::Type,
    mass:   Mass,
    #[builder(default = Volume { volume: <_>::default() })]
    volume: Volume,
}

/// Mass of a fluid type in a container.
#[derive(Component)]
pub struct Mass {
    pub mass: units::Mass,
}

/// The current volume occupied by a fluid type in a container.
#[derive(Component)]
pub struct Volume {
    pub volume: units::Volume,
}
