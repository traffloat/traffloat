//! Manages factory building logic.

use legion::Entity;
use smallvec::SmallVec;

use crate::cargo;
use crate::config::Id;
use crate::units;
use crate::SetupEcs;

/// A component attached to buildings that can perform reactions.
#[derive(getset::Getters)]
pub struct Factory {
    /// List of reactions supported by this factory.
    #[getset(get = "pub")]
    reactions: SmallVec<[Entity; 2]>,
}

/// A component attached to buildings which serve as factories.
#[derive(getset::Getters, getset::CopyGetters)]
pub struct Reaction {
    /// The node entity in which the reaction occurs.
    #[getset(get_copy = "pub")]
    node: Entity,
    /// The catalysts required for the reaction to take place.
    #[getset(get = "pub")]
    catalysts: SmallVec<[Catalyst; 1]>,
    /// Inputs and outputs to the reaction.
    #[getset(get = "pub")]
    puts: SmallVec<[Put; 2]>,
}

/// Wrapper for factory resources
pub mod resource {
    use super::*;

    /// This reaction requires cargo.
    #[derive(Debug, Clone, Copy, getset::CopyGetters)]
    pub struct Cargo<const N: usize> {
        /// The type of cargo required
        #[getset(get_copy = "pub")]
        ty: Id<cargo::Cargo>,
        /// The required sizes of the cargo
        #[getset(get_copy = "pub")]
        sizes: [units::CargoSize; N],
    }
    /// This reaction requires liquid.
    #[derive(Debug, Clone, Copy, getset::CopyGetters)]
    pub struct Liquid<const N: usize> {
        // TODO
    }
    /// This reaction requires gas.
    #[derive(Debug, Clone, Copy, getset::CopyGetters)]
    pub struct Gas<const N: usize> {
        // TODO
    }
    /// This reaction requires power.
    #[derive(Debug, Clone, Copy, getset::CopyGetters)]
    pub struct Power<const N: usize> {
        // TODO
    }
    /// This reaction requires sunlight.
    #[derive(Debug, Clone, Copy, getset::CopyGetters)]
    pub struct Light<const N: usize> {
        /// The required brightness values
        #[getset(get_copy = "pub")]
        brightness: [units::Brightness; N],
    }
    /// This reaction requires an operator with the specified skill.
    #[derive(Debug, Clone, Copy, getset::CopyGetters)]
    pub struct Skill<const N: usize> {
        // TODO
    }

    /// A type of resource whose existence affects a reaction.
    #[derive(Debug, Clone, Copy)]
    pub enum Conditional<const N: usize> {
        /// Existence of cargo
        Cargo(Cargo<{ N }>),
        /// Existence of liquid
        Liquid(Liquid<{ N }>),
        /// Existence of gas
        Gas(Gas<{ N }>),
        /// Existence of power
        Power(Power<{ N }>),
        /// Existence of light
        Light(Light<{ N }>),
        /// Existence of skilled operators
        Skill(Skill<{ N }>),
    }

    /// A type of resource that can be consumed.
    #[derive(Debug, Clone, Copy)]
    pub enum Consumable<const N: usize> {
        /// Consumed or generated cargo
        Cargo(Cargo<{ N }>),
        /// Consumed or generated liquid
        Liquid(Liquid<{ N }>),
        /// Consumed or generated gas
        Gas(Gas<{ N }>),
        /// Consumed or generated power
        Power(Power<{ N }>),
    }
}

/// A condition for a reaction.
#[derive(Debug, Clone, Copy, getset::CopyGetters)]
pub struct Catalyst {
    /// The levels at which the reaction takes place at minimum or maximum rate.
    #[getset(get_copy = "pub")]
    levels: resource::Conditional<2>,
    /// The multipliers when the reaction takes place at minimum or maximum rate..
    #[getset(get_copy = "pub")]
    multiplier: [f64; 2],
    /// The multiplier when the level is below minimum.
    #[getset(get_copy = "pub")]
    underflow_multiplier: f64,
    /// The multiplier when the level is above maximum.
    #[getset(get_copy = "pub")]
    overflow_multiplier: f64,
}

/// An input or output to a reaction.
#[derive(Debug, Clone, Copy, getset::CopyGetters)]
pub struct Put {
    /// The rate at which this conumsable is changed.
    #[getset(get_copy = "pub")]
    rate: resource::Consumable<1>, // TODO change to Rate<Consumable>
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup
}
