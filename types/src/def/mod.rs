//! Defines the mechanism of a game.

use derive_new::new;
use serde::{Deserialize, Serialize};

pub mod building;
pub mod cargo;
pub mod crime;
pub mod gas;
pub mod liquid;
pub mod reaction;
pub mod skill;
pub mod vehicle;

/// The map type used for ID-to-instance resolution.
pub type Map<K, V> = indexmap::IndexMap<K, V, fxhash::FxBuildHasher>;

/// Game mechanism definition.
#[derive(Debug, Clone, new, getset::Getters, getset::MutGetters, Serialize, Deserialize)]
pub struct GameDefinition {
    /// Cargo types.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    cargo: Map<cargo::TypeId, cargo::Type>,
    /// Cargo categories.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    cargo_cats: Map<cargo::CategoryId, cargo::Category>,
    /// Liquid types.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    liquid: Map<liquid::TypeId, liquid::Type>,
    /// Gas types.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    gas: Map<gas::TypeId, gas::Type>,
    /// Skill types.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    skill: Map<skill::TypeId, skill::Type>,
    /// Vehicle types.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    vehicle: Map<vehicle::TypeId, vehicle::Type>,
    /// Reaction types.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    reaction: Map<reaction::TypeId, reaction::Type>,
    /// Building types.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    building: Map<building::TypeId, building::Type>,
    /// Building categories.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    building_cats: Map<building::CategoryId, building::Category>,
    /// List of possible crimes.
    #[new(default)]
    #[getset(get = "pub", get_mut = "pub")]
    crime: Map<crime::TypeId, crime::Type>,
}
