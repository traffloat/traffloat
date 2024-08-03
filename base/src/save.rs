//! Save framework for Traffloat.
//!
//! The save format mainly consists of typed "definition" objects,
//! which mostly correspond to each effective entity in the world.
//!
//! To add a new persisted type, implement [`Def`] and add a new [`add_def`] definition.
//!
//! # Save format
//! There are two formats, msgpack and YAML.
//!
//! ## Msgpack
//! Msgpack is the normal save format to persist a world.
//! It starts with the [`MSGPACK_HEADER`], followed by a DEFLATE-encoded Msgpack buffer.
//! The data for each definition are stored as a separate Msgpack-encoded byte array
//! that deserializes to `Vec<Def>` of a fixed type.
//!
//! ## YAML
//! YAML is mostly used to create hand-written scenarios published through version control.
//! However, it is recommended to generate this YAML file with other tools instead.
//! YAML is used mainly due to cross-language compatibility.

#![allow(clippy::module_name_repetitions)]

use std::marker::PhantomData;

use bevy::app::{self, App};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Header line for YAML saves.
pub const YAML_HEADER: &[u8] = b"# $schema=https://traffloat.github.io/save.yaml\n";
/// Header bytes for Msgpack saves.
pub const MSGPACK_HEADER: &[u8] = b"\xFFtraffloat.github.io/save.msgpack\n";

/// Registers a new definition type to the app.
pub fn add_def<D: Def>(app: &mut App) {
    store::add_def::<D>(app);
    load::add_def::<D>(app);
}

mod load;
pub use load::{Depend as LoadDepend, LoadCommand, LoadFn, LoadOnce, LoadResult};

mod store;
pub use store::{
    Depend as StoreDepend, Depends as StoreDepends, StoreCommand, StoreResult, StoreSystem,
    StoreSystemFn, Writer,
};

#[cfg(test)]
mod tests;

/// Initializes the save framework.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) { app.add_plugins((store::Plugin, load::Plugin)); }
}

/// A type of save entry.
pub trait Def: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// A persistent identifier that indicates how to interpret a list of definitions.
    const TYPE: &'static str;

    /// Returns a system that converts world entities and resources into save data.
    ///
    /// Typically implemented by contsructing a [`StoreSystemFn`] with a system function.
    fn store_system() -> impl StoreSystem<Def = Self>;

    /// Returns a function that loads save data into the world.
    ///
    /// Typically implemented by contsructing a [`LoadFn`] with a system function.
    fn loader() -> impl LoadOnce<Def = Self>;
}

/// Identifies another save entry of type `D`.
///
/// `D` must be stored/loaded before the current type.
/// If self-dependency or cyclic dependency is required,
/// separate the logic to another save entry type instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id<D: Def>(u32, PhantomData<fn() -> D>);

impl<D: Def> Serialize for Id<D> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, D: Def> Deserialize<'de> for Id<D> {
    fn deserialize<De>(deserializer: De) -> Result<Self, De::Error>
    where
        De: serde::Deserializer<'de>,
    {
        u32::deserialize(deserializer).map(|id| Self(id, PhantomData))
    }

    fn deserialize_in_place<De>(deserializer: De, place: &mut Self) -> Result<(), De::Error>
    where
        De: serde::Deserializer<'de>,
    {
        u32::deserialize_in_place(deserializer, &mut place.0)
    }
}

#[derive(Serialize, Deserialize)]
struct YamlFile {
    types: Vec<YamlTypedData>,
}

#[derive(Serialize, Deserialize)]
struct YamlTypedData {
    r#type: String,
    defs:   serde_yaml::Value,
}

#[derive(Serialize, Deserialize)]
struct MsgpackFile {
    types: Vec<MsgpackTypedData>,
}

#[derive(Serialize, Deserialize)]
struct MsgpackTypedData {
    r#type: String,
    defs:   Vec<u8>,
}

/// Save format to use.
///
/// See [module-level documentation](self) for more information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// The Msgpack save format.
    Msgpack,
    /// The YAML save format.
    Yaml,
}
