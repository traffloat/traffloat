use bevy::app::{App, Plugin};
use bevy::math::Vec2;

/// Used by logic that depends on whether the graph layout is 2D or 3D.
pub type Vector = Vec2;

// Framework modules
pub mod persist;
pub mod request;
pub mod view;

mod cleanup;
pub use crate::cleanup::{CleanupAppExt, CleanupHooks, WorldObject};

// Domain-specific modules
pub mod fluid;
pub mod graph;
pub mod reaction;
pub mod reactor;
pub mod resident;
pub mod vehicle;

// Domain-aware modules
pub mod generate;

#[cfg(any(rust_analyzer, doc))]
pub mod docs;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.add_plugins(cleanup::Plug);
        app.add_plugins(persist::Plug);
        app.add_plugins(view::Plug);
        app.add_plugins(graph::Plug);
        app.add_plugins(fluid::Plug);
        app.add_plugins(resident::Plug);
        app.add_plugins(vehicle::Plug);
        app.add_plugins(reactor::Plug);
        app.add_plugins(request::Plug);
    }
}
