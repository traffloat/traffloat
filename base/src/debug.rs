//! Debugging utilities.

use bevy::ecs::bundle;

/// Debug info for an entity.
#[derive(bundle::Bundle)]
pub struct Bundle {
    #[cfg(feature = "entity-names")]
    name: bevy::core::Name,
}

impl Bundle {
    /// Provide name info fo an entity.
    #[must_use]
    pub fn new(_name: &'static str) -> Bundle {
        Bundle {
            #[cfg(feature = "entity-names")]
            name:                                  bevy::core::Name::new(_name),
        }
    }
}
