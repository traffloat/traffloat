//! Saves the JSON schema for save defs.

use std::borrow::Cow;

use bevy::app::{self, App};
use bevy::ecs::system::Resource;
use bevy::utils::HashMap;
use schemars::gen::SchemaGenerator;
use schemars::schema::Schema;

use super::Def;

pub(crate) fn add_def<D: Def>(app: &mut App) {
    app.world_mut().resource_mut::<Store>().0.insert(D::TYPE, json_schema_dyn::<D>());
}

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) { app.init_resource::<Store>(); }
}

/// Schema of all registered definitions.
#[derive(Default, Resource)]
pub struct Store(pub HashMap<&'static str, JsonSchemaDyn>);

fn json_schema_dyn<D: Def>() -> JsonSchemaDyn {
    JsonSchemaDyn {
        ty:               D::TYPE,
        is_referenceable: D::is_referenceable(),
        schema_name:      D::schema_name(),
        schema_id:        D::schema_id(),
        json_schema:      D::json_schema,
    }
}

/// vtable for [`JsonSchema`](schemars::JsonSchema).
#[allow(missing_docs)]
#[derive(Clone)]
pub struct JsonSchemaDyn {
    pub ty:               &'static str,
    pub is_referenceable: bool,
    pub schema_name:      String,
    pub schema_id:        Cow<'static, str>,
    pub json_schema:      fn(&mut SchemaGenerator) -> Schema,
}
