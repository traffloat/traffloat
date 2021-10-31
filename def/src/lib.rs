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

use std::convert::TryInto;
use std::io::Write;

use anyhow::Context;
use arcstr::ArcStr;
use codegen::Definition;
use getset::{CopyGetters, Getters, MutGetters};
use serde::{Deserialize, Serialize};
use traffloat_types::{time, units};
use typed_builder::TypedBuilder;

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
pub mod state;
pub mod vehicle;

/// The scenario schema version.
///
/// This value is only bumped when necessary to distinguish incompatible formats.
pub const SCHEMA_VERSION: u32 = 1;

/// The scenario magic header.
///
/// This value is only bumped when necessary to distinguish incompatible formats.
pub const MAGIC_HEADER: &[u8] = b"\xffTSV";

/// The schema for the binary save file.
#[derive(Getters, MutGetters, TypedBuilder, Serialize, Deserialize)]
pub struct Schema {
    /// Scenario metadata.
    #[getset(get = "pub")]
    scenario: Scenario,
    /// Scalar configuration for this scenario.
    #[getset(get = "pub")]
    config:   Config,
    /// All gamerule definitions.
    #[getset(get = "pub")]
    def:      Vec<Def>,
    /// State of game objects.
    #[getset(get = "pub")]
    #[getset(get_mut = "pub")]
    state:    state::State,
}

impl Schema {
    /// Parses a scenario file.
    pub fn parse(mut buf: &[u8]) -> anyhow::Result<Self> {
        buf = match buf.strip_prefix(MAGIC_HEADER) {
            Some(buf) => buf,
            _ => anyhow::bail!("Not a traffloat scenario file"),
        };
        let version = match buf.get(0..4) {
            Some(bytes) => u32::from_le_bytes(bytes.try_into().expect("bytes.len() == 4")),
            None => anyhow::bail!("File is too short"),
        };
        anyhow::ensure!(version == SCHEMA_VERSION, "Incompatible scenario version");

        let flate = flate2::read::DeflateDecoder::new(buf);
        rmp_serde::from_read(flate).context("Error parsing scenario file")
    }

    /// Writes a scenario file.
    pub fn write(&self, mut w: impl Write) -> anyhow::Result<()> {
        w.write_all(MAGIC_HEADER)?;
        w.write_all(&SCHEMA_VERSION.to_le_bytes())?;

        {
            use safety::Safety;

            let mut flate = flate2::write::DeflateEncoder::new(&mut w, flate2::Compression::best());
            self.serialize(&mut rmp_serde::Serializer::new(&mut flate))?;
            flate.flush()?;
            log::debug!(
                "Compressed scenario file ({}%)",
                flate.total_out().small_float() / flate.total_in().small_float() * 100.
            );
        }

        Ok(())
    }
}

/// Metadata for a scenario.
#[derive(Debug, Clone, Getters, Serialize, Deserialize)]
pub struct Scenario {
    /// Name of the scenario.
    #[getset(get = "pub")]
    name:        ArcStr,
    /// Description string of the scenario.
    #[getset(get = "pub")]
    description: ArcStr,
}

/// Scalar config for the scenario.
#[derive(Debug, Clone, CopyGetters, Serialize, Deserialize)]
pub struct Config {
    /// The angle the sun moves per tick
    #[getset(get_copy = "pub")]
    sun_speed:         time::Rate<f64>,
    /// The threshold below which liquid storages are considered zero.
    #[getset(get_copy = "pub")]
    negligible_volume: units::LiquidVolume,
}

/// Defines a game rule.
#[derive(Debug, Clone, Serialize, Deserialize, Definition)]
#[serde(tag = "type")]
#[resolve_context()]
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
    DefaultLiquidFormula(liquid::DefaultFormula),
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

#[cfg(feature = "convert-human-friendly")]
impl DefHumanFriendly {
    /// Returns the type ID for the wrapped type.
    pub fn value_type_id(&self) -> Option<std::any::TypeId> {
        use std::any::TypeId;

        Some(match self {
            Self::LangBundle(_) => TypeId::of::<lang::Def>(),
            Self::Atlas(_) => TypeId::of::<atlas::Def>(),
            Self::Liquid(_) => TypeId::of::<liquid::Def>(),
            Self::LiquidFormula(_) => return None,
            Self::DefaultLiquidFormula(_) => return None,
            Self::Gas(_) => TypeId::of::<gas::Def>(),
            Self::CargoCategory(_) => TypeId::of::<cargo::category::Def>(),
            Self::Cargo(_) => TypeId::of::<cargo::Def>(),
            Self::Skill(_) => TypeId::of::<skill::Def>(),
            Self::Vehicle(_) => TypeId::of::<vehicle::Def>(),
            Self::BuildingCategory(_) => TypeId::of::<building::category::Def>(),
            Self::Building(_) => TypeId::of::<building::Def>(),
            Self::Crime(_) => TypeId::of::<crime::Def>(),
        })
    }

    /// Returns the type ID for the wrapped type.
    pub fn value_type_name(&self) -> Option<&'static str> {
        use std::any::type_name;

        Some(match self {
            Self::LangBundle(_) => type_name::<lang::Def>(),
            Self::Atlas(_) => type_name::<atlas::Def>(),
            Self::Liquid(_) => type_name::<liquid::Def>(),
            Self::LiquidFormula(_) => return None,
            Self::DefaultLiquidFormula(_) => return None,
            Self::Gas(_) => type_name::<gas::Def>(),
            Self::CargoCategory(_) => type_name::<cargo::category::Def>(),
            Self::Cargo(_) => type_name::<cargo::Def>(),
            Self::Skill(_) => type_name::<skill::Def>(),
            Self::Vehicle(_) => type_name::<vehicle::Def>(),
            Self::BuildingCategory(_) => type_name::<building::category::Def>(),
            Self::Building(_) => type_name::<building::Def>(),
            Self::Crime(_) => type_name::<crime::Def>(),
        })
    }

    /// Returns the human-friendly string ID of the def.
    pub fn id_str(&self) -> Option<&ArcStr> {
        Some(match self {
            Self::LangBundle(def) => &def.id,
            Self::Atlas(def) => &def.id,
            Self::Liquid(def) => &def.id,
            Self::LiquidFormula(_) => return None,
            Self::DefaultLiquidFormula(_) => return None,
            Self::Gas(def) => &def.id,
            Self::CargoCategory(def) => &def.id,
            Self::Cargo(def) => &def.id,
            Self::Skill(def) => &def.id,
            Self::Vehicle(def) => &def.id,
            Self::BuildingCategory(def) => &def.id,
            Self::Building(def) => &def.id,
            Self::Crime(def) => &def.id,
        })
    }
}

/// A customizable name that is either a translation or a value from user input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomizableName {
    /// An original item from a lang file.
    Original(lang::Item),
    /// A custom name from user input.
    Custom(ArcStr),
}

#[cfg(feature = "yew")]
impl<'t> yew::html::ImplicitClone for CustomizableName {}

#[cfg(feature = "yew")]
impl<'t> yew::html::IntoPropValue<CustomizableName> for &'t lang::Item {
    fn into_prop_value(self) -> CustomizableName { CustomizableName::Original(self.clone()) }
}

#[cfg(feature = "yew")]
impl<'t> yew::html::IntoPropValue<CustomizableName> for &'t str {
    fn into_prop_value(self) -> CustomizableName { CustomizableName::Custom(ArcStr::from(self)) }
}
