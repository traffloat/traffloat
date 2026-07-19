//! This module handles high-level vehicle motion in and between buildings and rails.
//! determining whether to accelerate or deceelerate based on vehicle motion rules.
//!
//! In particular, this module is responsible for controlling braking
//! with the primary purpose to avoid collision.
//!
//! # Input
//! The driver resident AI executes pathfinding on its own,
//! generating instructions in the form of [`Intent`].
//!
//! # Output
//! The motion plugin updates [`super::propulsion::Desired`],
//! which is executed by the propulsion plugin to actually execute the motion plan.

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::reflect::Reflect;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) { app.register_type::<Intent>(); }
}

/// Input for the motion plugin,
/// indicating the next step of motion to prepare for braking or reservation.
#[derive(Component, Reflect, Default)]
pub enum Intent {
    #[default]
    None,
    Building {
        target: Entity,
    },
    Rail {
        target_rail:      Entity,
        through_building: Entity,
    },
}

// TODO control loop
