//! Reaction definitions

use std::ops::Range;

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use typed_builder::TypedBuilder;

use super::{cargo, gas, liquid, skill};
use crate::time::Rate;
use crate::units;

/// Identifies a reaction type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeId(pub usize);

/// A type of reaction.
#[derive(
    Debug, Clone, TypedBuilder, getset::CopyGetters, getset::Getters, Serialize, Deserialize,
)]
pub struct Type {
    /// Name of the reaction type.
    #[getset(get = "pub")]
    name: ArcStr,
    /// Description of the reaction type.
    #[getset(get = "pub")]
    description: ArcStr,
    /// Catalysts for the reaction.
    #[getset(get = "pub")]
    catalysts: SmallVec<[Catalyst; 2]>,
    /// Inputs and outputs for the reaction.
    #[getset(get = "pub")]
    puts: SmallVec<[Put; 2]>,
}

/// A condition or catalyst which affects the rate of a reaction.
#[derive(
    Debug, Clone, TypedBuilder, getset::Getters, getset::CopyGetters, Serialize, Deserialize,
)]
pub struct Catalyst {
    /// The lerp endpoints of the catalyst.
    #[getset(get = "pub")]
    range: CatalystRange,
    /// The multipliers associated with the catalyst.
    #[getset(get_copy = "pub")]
    multipliers: Multipliers,
}

/// A type of resource whose existence affects a reaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CatalystRange {
    /// Existence of cargo
    Cargo {
        /// Type of cargo catalyst
        ty: cargo::TypeId,
        /// Min and max levels of cargo catalyst
        levels: Range<units::CargoSize>,
    },
    /// Existence of liquid
    Liquid {
        /// Type of liquid catalyst
        ty: liquid::TypeId,
        /// Min and max levels of liquid catalyst
        levels: Range<units::LiquidVolume>,
    },
    /// Existence of gas
    Gas {
        /// Type of gas catalyst
        ty: gas::TypeId,
        /// Min and max levels of gas catalyst
        levels: Range<units::GasVolume>,
    },
    /// Existence of power
    Electricity {
        /// Min and max levels of electricity catalyst
        levels: Range<units::ElectricPower>,
    },
    /// Existence of light
    Light {
        /// Min and max levels of light catalyst
        levels: Range<units::Brightness>,
    },
    /// Existence of skilled operators
    ///
    /// Only the most skilled operator is counted as a catalyst.
    Skill {
        /// Type of skill catalyst
        ty: skill::TypeId,
        /// Min and max levels of skill catalyst
        levels: Range<units::Skill>,
    },
}

/// The multipliers associated with a catalyst.
#[derive(Debug, Clone, Copy, TypedBuilder, getset::CopyGetters, Serialize, Deserialize)]
pub struct Multipliers {
    /// Multiplier to the reaction rate when the catalyst is in deficiency.
    #[getset(get_copy = "pub")]
    underflow: f64,
    /// Multiplier to the reaction rate when the catalyst is at the min lerp endpoint.
    #[getset(get_copy = "pub")]
    min: f64,
    /// Multiplier to the reaction rate when the catalyst is at the max lerp endpoint.
    #[getset(get_copy = "pub")]
    max: f64,
    /// Multiplier to the reaction rate when the catalyst is in excess.
    #[getset(get_copy = "pub")]
    overflow: f64,
}

/// The inputs and outputs of a reaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Put {
    /// Consumption or production of cargo
    Cargo {
        /// Type of cargo consumed/produced
        ty: cargo::TypeId,
        /// Base (unmultiplied) rate of gas consumed/produced
        base: Rate<units::CargoSize>,
    },
    /// Consumption or production of liquid
    Liquid {
        /// Type of liquid consumed/produced
        ty: liquid::TypeId,
        /// Base (unmultiplied) rate of liquid consumed/produced
        base: Rate<units::LiquidVolume>,
    },
    /// Consumption or production of gas
    Gas {
        /// Type of gas consumed/produced
        ty: gas::TypeId,
        /// Base (unmultiplied) rate of gas consumed/produced
        base: Rate<units::GasVolume>,
    },
    /// Consumption or generation or electricity
    Electricity {
        /// Base (unmultiplied) rate of electricity consumed/generated
        base: Rate<units::ElectricPower>,
    },
    /// Change in skill
    ///
    /// Operators used as catalyst do not receive the skill change.
    /// All other operators receive the same amount of change.
    Skill {
        /// Type of skill trained/lost
        ty: skill::TypeId,
        /// Base (unmultiplied) rate of gas consumed/produced
        base: Rate<units::Skill>,
    },
}

impl Put {
    /// Base put rate of the resource.
    fn base(&self) -> f64 {
        match self {
            Self::Cargo { base, .. } => base.0.value(),
            Self::Liquid { base, .. } => base.0.value(),
            Self::Gas { base, .. } => base.0.value(),
            Self::Electricity { base, .. } => base.0.value(),
            Self::Skill { base, .. } => base.0.value(),
        }
    }

    /// Whether this is an output
    pub fn is_output(&self) -> bool {
        self.base() > 0.
    }

    /// Whether this is an input
    pub fn is_input(&self) -> bool {
        self.base() < 0.
    }
}
