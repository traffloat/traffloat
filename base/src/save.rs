use std::marker::PhantomData;

use bevy::app::App;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn add_def<D: Def>(app: &mut App) { store::add_def::<D>(app); }

pub mod load;
pub mod store;

pub trait Def: Serialize + DeserializeOwned + prost::Message + 'static {
    const TYPE: &'static str;

    fn store_system() -> impl store::StoreSystem<Def = Self>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id<D: Def>(usize, PhantomData<fn() -> D>);

struct YamlTypedData {
    r#type: String,
    defs:   Vec<serde_yaml::Value>,
}

#[derive(prost::Message)]
struct ProtobufTypedData {
    #[prost(string, tag = "1")]
    r#type: String,
    #[prost(message, repeated, tag = "2")]
    defs:   Vec<prost_types::Any>,
}
