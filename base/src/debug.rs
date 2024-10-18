//! Debugging utilities.

use std::borrow::Cow;

use bevy::ecs::bundle;

/// Debug info for an entity.
#[derive(bundle::Bundle)]
pub struct Bundle {
    #[cfg(feature = "entity-names")]
    name: bevy::core::Name,
}

impl Bundle {
    /// Provide name info of an entity.
    #[must_use]
    pub fn new(name: &'static str) -> Bundle { Self::new_with(|| name) }

    /// Provide name info of an entity on demand.
    #[must_use]
    pub fn new_with<Name: Into<Cow<'static, str>>>(_name: impl FnOnce() -> Name) -> Bundle {
        Bundle {
            #[cfg(feature = "entity-names")]
            name:                                  bevy::core::Name::new(_name()),
        }
    }
}
