//! Reaction definitions

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use traffloat_types::time::Rate;
use traffloat_types::units;

use crate::catalyst::Catalyst;
use crate::{cargo, gas, lang, liquid, skill};

/// A type of reaction.
#[derive(Debug, Clone, CopyGetters, Getters, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct Reaction {
    /// Title for the reaction.
    #[getset(get = "pub")]
    title:       lang::Item,
    /// Description for the reaction.
    #[getset(get = "pub")]
    description: lang::Item,
    /// Catalysts for the reaction.
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    catalysts:   SmallVec<[Catalyst; 2]>,
    /// Inputs and outputs for the reaction.
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    puts:        SmallVec<[Put; 2]>,
    /// Policies for the reaction.
    #[getset(get = "pub")]
    policy:      Policy,
}

/// The inputs and outputs of a reaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize), serde(tag = "type")))]
pub enum Put {
    /// Consumption or production of cargo
    Cargo {
        /// Type of cargo consumed/produced
        ty:   cargo::Id,
        /// Base (unmultiplied) rate of gas consumed/produced
        base: Rate<units::CargoSize>,
    },
    /// Consumption or production of liquid
    Liquid {
        /// Type of liquid consumed/produced
        ty:   liquid::Id,
        /// Base (unmultiplied) rate of liquid consumed/produced
        base: Rate<units::LiquidVolume>,
    },
    /// Consumption or production of gas
    Gas {
        /// Type of gas consumed/produced
        ty:   gas::Id,
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
        ty:   skill::Id,
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
    pub fn is_output(&self) -> bool { self.base() > 0. }

    /// Whether this is an input
    pub fn is_input(&self) -> bool { self.base() < 0. }
}

/// Reaction behaviour specific to this building.
#[derive(Debug, Clone, CopyGetters, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct Policy {
    /// Whethre the reaction rate can be configured by the players.
    #[get_copy = "pub"]
    configurable: bool,
    /// What happens when inputs underflow.
    #[get_copy = "pub"]
    on_underflow: FlowPolicy,
    /// What happens when outputs overflow.
    #[get_copy = "pub"]
    on_overflow:  FlowPolicy,
}

/// behaviour when inputs underflow or outputs overflow.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize), serde(tag = "type")))]
pub enum FlowPolicy {
    /// Reduce the rate of reaction such that the input/output capacity is just enough.
    ReduceRate,
}

impl Default for FlowPolicy {
    fn default() -> Self { Self::ReduceRate }
}
