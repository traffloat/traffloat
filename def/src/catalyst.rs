//! A condition or catalyst which affects the efficiency of a feature.

use std::ops::Range;

use getset::CopyGetters;
use serde::{Deserialize, Serialize};
use traffloat_types::units;

use crate::{building, cargo, gas, liquid, skill};

/// A condition or catalyst.
#[derive(Debug, Clone, getset::Getters, getset::CopyGetters, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct Catalyst {
    /// The lerp endpoints of the catalyst.
    #[getset(get = "pub")]
    range:       CatalystRange,
    /// The multipliers associated with the catalyst.
    #[getset(get_copy = "pub")]
    multipliers: Multipliers,
}

/// A type of resource whose existence affects a reaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize), serde(tag = "type")))]
pub enum CatalystRange {
    /// Existence of cargo.
    Cargo {
        /// Type of cargo catalyst
        ty:     cargo::Id,
        /// Min and max levels of cargo catalyst
        levels: Range<units::CargoSize>,
    },
    /// Existence of liquid.
    Liquid {
        /// Type of liquid catalyst
        ty:     liquid::Id,
        /// Min and max levels of liquid catalyst
        levels: Range<units::LiquidVolume>,
    },
    /// Existence of gas.
    Gas {
        /// Type of gas catalyst
        ty:     gas::Id,
        /// Min and max levels of gas catalyst
        levels: Range<units::GasVolume>,
    },
    /// Existence of power.
    Electricity {
        /// Min and max levels of electricity catalyst
        levels: Range<units::ElectricPower>,
    },
    /// Existence of light.
    Light {
        /// Min and max levels of light catalyst
        levels: Range<units::Brightness>,
    },
    /// Existence of skilled inhabitants.
    ///
    /// This is computed by the sum of skill level of all inhabitants
    /// in the specified population storage.
    Skill {
        /// Type of skill catalyst
        ty:      skill::Id,
        /// Min and max levels of skill catalyst
        levels:  Range<units::Skill>,
        /// The population storage to compute skills from.
        storage: building::storage::population::Id,
    },
}

/// The multipliers associated with a catalyst.
#[derive(Debug, Clone, Copy, CopyGetters, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct Multipliers {
    /// Multiplier to the reaction rate when the catalyst is in deficiency.
    #[getset(get_copy = "pub")]
    underflow: f64,
    /// Multiplier to the reaction rate when the catalyst is at the min lerp endpoint.
    #[getset(get_copy = "pub")]
    min:       f64,
    /// Multiplier to the reaction rate when the catalyst is at the max lerp endpoint.
    #[getset(get_copy = "pub")]
    max:       f64,
    /// Multiplier to the reaction rate when the catalyst is in excess.
    #[getset(get_copy = "pub")]
    overflow:  f64,
}
