//! Save framework for Traffloat.
//!
//! The save format mainly consists of typed "definition" objects,
//! which mostly correspond to each effective entity in the world.
//!
//! To add a new persisted type, implement [`Def`] and add a new [`add_def`] definition.
//!
//! # Save format
//! There are two formats, msgpack and JSON.
//!
//! ## Msgpack
//! Msgpack is the normal save format to persist a world.
//! It starts with the [`MSGPACK_HEADER`], followed by a DEFLATE-encoded Msgpack buffer.
//! The data for each definition are stored as a separate Msgpack-encoded byte array
//! that deserializes to `Vec<Def>` of a fixed type.
//!
//! ## JSON
//! JSON is mostly used to create hand-written scenarios published through version control.
//! However, it is recommended to generate this JSON file with other tools instead.
//! JSON is used mainly due to cross-language compatibility.

#![allow(clippy::module_name_repetitions)]

use std::any::type_name;
use std::borrow::Cow;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;

use bevy::app::{self, App};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Header bytes for Msgpack saves.
pub const MSGPACK_HEADER: &[u8] = b"\xFFtraffloat.github.io/save.msgpack\n";

/// Registers a new definition type to the app.
pub fn add_def<D: Def>(app: &mut App) {
    store::add_def::<D>(app);
    load::add_def::<D>(app);

    #[cfg(feature = "schema")]
    schema::add_def::<D>(app);
}

#[cfg(feature = "schema")]
pub mod schema;

mod load;
pub use load::{Depend as LoadDepend, LoadCommand, LoadFn, LoadOnce, LoadResult};

mod store;
use serde_json::value::RawValue;
pub use store::{
    Depend as StoreDepend, Depends as StoreDepends, StoreCommand, StoreResult, StoreSystem,
    StoreSystemFn, Writer,
};

#[cfg(test)]
mod tests;

/// Initializes the save framework.
pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((store::Plugin, load::Plugin));
        #[cfg(feature = "schema")]
        app.add_plugins(schema::Plugin);
    }
}

/// A type of save entry.
pub trait Def: Serialize + DeserializeOwned + cfg_schema::Bound + Send + Sync + 'static {
    /// A persistent identifier that indicates how to interpret a list of definitions.
    const TYPE: &'static str;

    /// The runtime type that maps to this definition,
    /// e.g. an `Entity` referencing the entity saved by this entry.
    type Runtime: fmt::Debug + Copy + PartialEq + Eq + Hash + Send + Sync;

    /// Returns a system that converts world entities and resources into save data.
    ///
    /// Typically implemented by contsructing a [`StoreSystemFn`] with a system function.
    fn store_system() -> impl StoreSystem<Def = Self>;

    /// Returns a function that loads save data into the world.
    ///
    /// Typically implemented by contsructing a [`LoadFn`] with a system function.
    fn loader() -> impl LoadOnce<Def = Self>;
}

#[cfg(feature = "schema")]
mod cfg_schema {
    use schemars::JsonSchema;

    pub trait Bound: JsonSchema {}
    impl<T: JsonSchema> Bound for T {}
}

#[cfg(not(feature = "schema"))]
mod cfg_schema {
    pub trait Bound {}
    impl<T> Bound for T {}
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

#[cfg(feature = "schema")]
const _: () = {
    use schemars::gen::SchemaGenerator;
    use schemars::schema::Schema;

    impl<D: Def> schemars::JsonSchema for Id<D> {
        fn schema_name() -> String { format!("Id_of_{}", type_name::<D>()) }

        fn schema_id() -> Cow<'static, str> {
            Cow::Owned(format!("traffloat_base::save::Id<{}>", type_name::<D>()))
        }

        fn is_referenceable() -> bool { false }

        fn json_schema(gen: &mut SchemaGenerator) -> Schema {
            <u32 as schemars::JsonSchema>::json_schema(gen)
        }
    }
};

/// Schema of a JSON save file.
#[derive(Serialize, Deserialize)]
pub struct JsonFile {
    types: Vec<JsonTypedData>,
}

/// A group of homogeneous entries in a JSON save file.
#[derive(Serialize, Deserialize)]
pub struct JsonTypedData {
    r#type: String,
    defs:   Box<RawValue>,
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
    /// The JSON save format.
    Json,
}
