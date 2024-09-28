//! Fluid definitions.

mod scalar;
mod types;

use bevy::app::{self, App};
pub use scalar::{Save as SaveScalar, Scalar};
use traffloat_base::save;
pub use types::{create_type, CreatedType, OnCreateType, Save as SaveType, Type, TypeDef, Types};

/// Initializes fluid simulation systems.
pub(super) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Scalar>();
        app.init_resource::<CreatedType>();
        save::add_def::<SaveScalar>(app);
        save::add_def::<SaveType>(app);
    }
}
