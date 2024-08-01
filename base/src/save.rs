use std::marker::PhantomData;

use bevy::app::{self, App};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub const YAML_HEADER: &[u8] = b"# $schema=https://traffloat.github.io/save.yaml\n";
pub const MSGPACK_HEADER: &[u8] = b"\xFFtraffloat.github.io/save.msgpack\n";

pub fn add_def<D: Def>(app: &mut App) {
    store::add_def::<D>(app);
    load::add_def::<D>(app);
}

pub mod load;
pub use load::{LoadCommand, LoadFn, LoadOnce, LoadResult};

mod store;
pub use store::{
    Depend as StoreDepend, Depends as StoreDepends, StoreCommand, StoreResult, StoreSystem,
    StoreSystemFn, Writer,
};

#[cfg(test)]
mod tests;

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) { app.add_plugins((store::Plugin, load::Plugin)); }
}

pub trait Def: Serialize + DeserializeOwned + Send + Sync + 'static {
    const TYPE: &'static str;

    fn store_system() -> impl StoreSystem<Def = Self>;

    fn loader() -> impl LoadOnce<Def = Self>;
}

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

pub enum Format {
    Yaml,
    Msgpack,
}
