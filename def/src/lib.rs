//! Defines the mechanism of a game scenario.

#![deny(
    anonymous_parameters,
    bare_trait_objects,
    clippy::clone_on_ref_ptr,
    clippy::float_cmp_const,
    clippy::if_not_else,
    clippy::unwrap_used
)]
#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, clippy::match_single_binding,)
)]
#![cfg_attr(any(doc, not(debug_assertions)), deny(missing_docs))]
#![cfg_attr(
    not(debug_assertions),
    deny(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::dbg_macro,
        clippy::indexing_slicing,
    )
)]

use arcstr::ArcStr;
use codegen::Definition;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use traffloat_types::{time, units};

pub mod atlas;
pub mod building;
pub mod cargo;
pub mod catalyst;
pub mod crime;
pub mod feature;
pub mod gas;
pub mod lang;
pub mod liquid;
pub mod skill;
pub mod vehicle;

/// Metadata for a scenario.
#[derive(Serialize, Deserialize, Getters)]
pub struct Scenario {
    /// Name of the scenario.
    #[getset(get = "pub")]
    name:        ArcStr,
    /// Description string of the scenario.
    #[getset(get = "pub")]
    description: ArcStr,
}

/// Scalar config for the scenario.
#[derive(Serialize, Deserialize, CopyGetters)]
pub struct Config {
    /// The angle the sun moves per tick
    #[getset(get_copy = "pub")]
    sun_speed:         time::Rate<f64>,
    /// The threshold below which liquid storages are considered zero.
    #[getset(get_copy = "pub")]
    negligible_volume: units::LiquidVolume,
}

/// Defines a game rule.
#[derive(Serialize, Deserialize, Definition)]
#[serde(tag = "type")]
pub enum Def {
    /// Defines a language bundle reference.
    LangBundle(lang::Def),
    /// Defines a texture atlas reference.
    Atlas(atlas::Def),
    /// Defines a liquid type.
    Liquid(liquid::Def),
    /// Defines a liquid formula.
    LiquidFormula(liquid::Formula),
    /// Defines the default liquid formula.
    DefaultLiquidFormula(liquid::Formula),
    /// Defines a gas type.
    Gas(gas::Def),
    /// Defines a category of cargo types.
    CargoCategory(cargo::category::Def),
    /// Defines a cargo type.
    Cargo(cargo::Def),
    /// Defines a skill.
    Skill(skill::Def),
    /// Defines a vehicle.
    Vehicle(vehicle::Def),
    /// Defines a category of building types.
    BuildingCategory(building::category::Def),
    /// Defines a building type.
    Building(building::Def),
    /// Defines a crime type.
    Crime(crime::Def),
}
