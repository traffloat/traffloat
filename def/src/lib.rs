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
use derive_new::new;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};
use traffloat_types::{time, units};
use typed_builder::TypedBuilder;
use xias::Xias;
#[cfg(feature = "xy")]
pub use xylem::Xylem;

macro_rules! impl_identifiable {
    ($ty:ty, $scope:ty) => {
        #[cfg(feature = "xy")]
        impl ::xylem::Identifiable<crate::Schema> for $ty {
            type Scope = $scope;

            fn id(&self) -> ::xylem::Id<crate::Schema, $ty> { self.id }
        }
    };
    ($ty:ty) => {
        impl_identifiable!($ty, ());
    };
}

pub mod atlas;
pub mod building;
pub mod cargo;
pub mod catalyst;
pub mod crime;
pub mod edge;
pub mod feature;
pub mod gas;
pub mod lang;
pub mod liquid;
pub mod node;
pub mod skill;
pub mod vehicle;

mod tests;

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
pub struct TfsaveFile {
    /// Scenario metadata.
    #[getset(get = "pub")]
    scenario: Scenario,
    /// Scalar configuration for this scenario.
    #[getset(get = "pub")]
    config:   Config,
    /// All gamerule definitions.
    #[getset(get = "pub")]
    def:      Vec<AnyDef>,
    /// State of game objects.
    #[getset(get = "pub")]
    #[getset(get_mut = "pub")]
    state:    State,
}

impl TfsaveFile {
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
        buf = buf.get(4..).expect("Just checked above");

        let flate = flate2::read::DeflateDecoder::new(buf);
        rmp_serde::from_read(flate).context("Error parsing scenario file")
    }

    /// Writes a scenario file.
    pub fn write(&self, mut w: impl Write) -> anyhow::Result<()> {
        w.write_all(MAGIC_HEADER)?;
        w.write_all(&SCHEMA_VERSION.to_le_bytes())?;

        {
            let mut flate = flate2::write::DeflateEncoder::new(&mut w, flate2::Compression::best());
            self.serialize(&mut rmp_serde::Serializer::new(&mut flate))?;
            flate.flush()?;
            log::debug!(
                "Compressed scenario file ({}%)",
                flate.total_out().small_float::<f64>() / flate.total_in().small_float::<f64>()
                    * 100.
            );
            flate.finish()?;
        }

        Ok(())
    }
}

/// Metadata for a scenario.
#[derive(Debug, Clone, Getters, Serialize, Deserialize, TypedBuilder)]
pub struct Scenario {
    /// Name of the scenario.
    #[getset(get = "pub")]
    name:        ArcStr,
    /// Description string of the scenario.
    #[getset(get = "pub")]
    description: ArcStr,
}

/// Scalar config for the scenario.
#[derive(Debug, Clone, CopyGetters, Serialize, Deserialize, TypedBuilder)]
pub struct Config {
    /// The angle the sun moves per tick
    #[getset(get_copy = "pub")]
    sun_speed:         time::Rate<f64>,
    /// The threshold below which liquid storages are considered zero.
    #[getset(get_copy = "pub")]
    negligible_volume: units::LiquidVolume,
}

mod schema;
#[cfg(feature = "xy")]
pub use schema::curdir;
pub use schema::{Id, IdString, Schema};

/// Defines a game rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[cfg_attr(feature = "xy", derive(Xylem))]
#[cfg_attr(feature = "xy", xylem(expose = AnyDefXylem, derive(Deserialize), serde(tag = "type")))]
pub enum AnyDef {
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

/// The state of objects in a game.
#[derive(Default, Getters, Setters, CopyGetters, MutGetters, Serialize, Deserialize, new)]
pub struct State {
    /// Current game time.
    #[getset(get_copy = "pub")]
    #[getset(set = "pub")]
    time:  time::Instant,
    /// State of all nodes in the game.
    #[new(default)]
    #[getset(get = "pub")]
    #[getset(get_mut = "pub")]
    nodes: Vec<node::Node>,
    /// State of all edges in the game.
    #[new(default)]
    #[getset(get = "pub")]
    #[getset(get_mut = "pub")]
    edges: Vec<edge::Edge>,
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
