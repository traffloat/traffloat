//! Common utility framework.

use bevy::app::{self, App};
use bevy::ecs::schedule::IntoSystemSetConfigs;

pub mod proto;
pub mod save;
mod state;
pub use state::EmptyState;
pub mod partition;
pub use partition::{
    ClientSideSystemSet, EventReaderSystemSet, EventWriterSystemSet, ServerSideSystemSet,
    UiMutatorSystemSet,
};
pub mod debug;

/// Register base configurations.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(app::Update, UiMutatorSystemSet.ambiguous_with(UiMutatorSystemSet));
        app.configure_sets(app::Update, ClientSideSystemSet.ambiguous_with(ServerSideSystemSet));
        app.add_plugins(save::Plugin);
    }
}
