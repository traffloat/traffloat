//! Saving game definition and state.

use std::convert::TryInto;

use cfg_if::cfg_if;
use legion::world::{ComponentError, SubWorld};
use legion::{Entity, EntityStore, IntoQuery};
use safety::Safety;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// The save schema version.
///
/// This value is only bumped when necessary to distinguish incompatible formats.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Definition {
    LangBundle(LangBundle),
    Atlas(Atlas),
    BuildingCategory(BuildingCategory),
    Building(Building),
    CargoCategory(CargoCategory),
    Cargo(Cargo),
}
