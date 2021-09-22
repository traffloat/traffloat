//! Defines the mechanism of a game.

use serde::{Deserialize, Serialize};

pub mod building;
pub mod cargo;
pub mod catalyst;
pub mod crime;
pub mod feature;
pub mod gas;
pub mod liquid;
pub mod skill;
pub mod vehicle;

/// The map type used for ID-to-instance resolution.
pub type Map<K, V> = indexmap::IndexMap<K, V, fxhash::FxBuildHasher>;

/// Game mechanism definition.
#[derive(Debug, Clone, getset::Getters, getset::MutGetters, Serialize, Deserialize)]
pub struct GameDefinition {
    /// Cargo types.
    #[getset(get = "pub", get_mut = "pub")]
    cargo: Map<cargo::TypeId, cargo::Type>,
    /// Cargo categories.
    #[getset(get = "pub", get_mut = "pub")]
    cargo_cats: Map<cargo::CategoryId, cargo::Category>,
    /// Liquid types.
    #[getset(get = "pub", get_mut = "pub")]
    liquid: Map<liquid::TypeId, liquid::Type>,
    /// Liquid mixing behaviour.
    #[getset(get = "pub", get_mut = "pub")]
    liquid_mixer: liquid::Mixer,
    /// Gas types.
    #[getset(get = "pub", get_mut = "pub")]
    gas: Map<gas::TypeId, gas::Type>,
    /// Skill types.
    #[getset(get = "pub", get_mut = "pub")]
    skill: Map<skill::TypeId, skill::Type>,
    /// Vehicle types.
    #[getset(get = "pub", get_mut = "pub")]
    vehicle: Map<vehicle::TypeId, vehicle::Type>,
    /// Building types.
    #[getset(get = "pub", get_mut = "pub")]
    building: Map<building::TypeId, building::Type>,
    /// Building categories.
    #[getset(get = "pub", get_mut = "pub")]
    building_cats: Map<building::CategoryId, building::Category>,
    /// List of possible crimes.
    #[getset(get = "pub", get_mut = "pub")]
    crime: Map<crime::TypeId, crime::Type>,
}
