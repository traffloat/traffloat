//! Reaction is a generic conversion process based on runtime-defined recipes.
//!
//! This module alone does not register any functionality directly.
//! Instead, it provides the reusable building blocks for defining reactions,
//! which can be fit into other plugins involving reactions, including:
//! - [`crate::reactor`] for facility-scoped reactions
//! - [`crate::resident::ambient`] for resident-scoped reactions
//! - [`crate::vehicle`] fuel consumption
//! <!-- - spontaneous fluid/cargo reactions -->

use bevy::ecs::entity::Entity;
use bevy::math::FloatExt;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::{fluid, resident};

pub mod catalyst;
pub mod input;
pub mod output;

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Threshold {
    /// The interpolation curve to use between the min and max input.
    pub curve:         Curve,
    /// How this threshold modifies the efficiency of the reaction.
    pub modifier_type: ThresholdModifierType,
}

impl Threshold {
    #[must_use]
    pub fn lerp(&self, input: f32) -> EfficiencyModifierResult {
        let out = match self.curve {
            Curve::Linear { min_input, max_input, min_multiplier, max_multiplier } => {
                let clamped_input = input.clamp(min_input, max_input);
                let t = (clamped_input - min_input) / (max_input - min_input);
                min_multiplier + t * (max_multiplier - min_multiplier)
            }
            Curve::Triangle {
                min_input,
                mid_input,
                max_input,
                start_multiplier,
                mid_multiplier,
                end_multiplier,
            } => {
                if input <= mid_input {
                    let t = (input - min_input).max(0.0) / (mid_input - min_input);
                    start_multiplier.lerp(mid_multiplier, t)
                } else {
                    let t = (max_input - input).max(0.0) / (max_input - mid_input);
                    end_multiplier.lerp(mid_multiplier, t)
                }
            }
            Curve::Gaussian {
                optimal_input,
                input_scale,
                optimal_multiplier,
                minimal_multiplier,
            } => {
                minimal_multiplier
                    + (optimal_multiplier - minimal_multiplier)
                        * (-(2.0 * (input - optimal_input) / input_scale).powi(2)).exp()
            }
        };
        match self.modifier_type {
            ThresholdModifierType::Multiplier => {
                EfficiencyModifierResult { multiplier: out, maximum: 1.0 }
            }
            ThresholdModifierType::Maximum => {
                EfficiencyModifierResult { multiplier: 1.0, maximum: out }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum ThresholdModifierType {
    /// This threshold multiplies the efficiency by the output of the curve.
    Multiplier,
    /// This threshold restricts efficiency from exceeding the output of the curve.
    Maximum,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum Curve {
    /// Linear slope within input range, constant beyond.
    Linear {
        /// Start X-coordinate of the interpolation curve.
        min_input:      f32,
        /// End X-coordinate of the interpolation curve.
        max_input:      f32,
        /// Start Y-coordinate of the interpolation curve.
        min_multiplier: f32,
        /// End Y-coordinate of the interpolation curve.
        max_multiplier: f32,
    },
    /// Two piecewise linear curves within input range, constant beyond.
    Triangle {
        /// Start X-coordinate of the first piece.
        min_input:        f32,
        /// X-coordinate where the first piece transitions to the second piece.
        mid_input:        f32,
        /// End X-coordinate of the second piece.
        max_input:        f32,
        /// Y-coordinate below and at `min_input`.
        start_multiplier: f32,
        /// Y-coordinate at `mid_input`.
        mid_multiplier:   f32,
        /// Y-coordinate above and at `max_input`.
        end_multiplier:   f32,
    },
    /// Gaussian curve, exactly optimal efficiency at `optimal_input`,
    /// symmetrically asymptoting towards minimal efficiency towards &pm;&infin;,
    /// passing at lerp(minimal, optimal, 1.83%) at `optimal_input` &pm; `input_scale`.
    Gaussian {
        /// The X-coordinate of the extremum of the Gaussian curve.
        optimal_input:      f32,
        /// Scales the input range.
        /// This is approximately 2.49 times of the standard deviation.
        input_scale:        f32,
        /// The Y-coordinate at the extremum of the Gaussian curve.
        optimal_multiplier: f32,
        /// The Y-coordinate as input goes to &pm;&infin;.
        minimal_multiplier: f32,
    },
}

/// A fold function to reduce zero or multiple float parameters into one.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum Aggregator {
    Sum,
    Product,
    Max { initial: f32 },
    Min { initial: f32 },
}

impl Aggregator {
    pub fn initial(&self) -> f32 {
        match *self {
            Aggregator::Sum => 0.0,
            Aggregator::Product => 1.0,
            Aggregator::Max { initial } | Aggregator::Min { initial } => initial,
        }
    }

    pub fn reduce(&self, acc: &mut f32, value: f32) {
        match self {
            Aggregator::Sum => *acc += value,
            Aggregator::Product => *acc *= value,
            Aggregator::Max { .. } => *acc = (*acc).max(value),
            Aggregator::Min { .. } => *acc = (*acc).min(value),
        }
    }
}

pub trait EfficiencyModifier<P, D> {
    fn compute_efficiency(&self, params: &P, data: &D) -> EfficiencyModifierResult;
}

#[derive(Debug, Clone, Copy)]
pub struct EfficiencyModifierResult {
    /// This modifier multiplies the efficiency by this value.
    pub multiplier: f32,
    /// This modifier restricts efficiency from exceeding this value.
    pub maximum:    f32,
}

impl Default for EfficiencyModifierResult {
    fn default() -> Self { Self::IDENTITY }
}

impl EfficiencyModifierResult {
    pub const IDENTITY: Self = Self { multiplier: 1.0, maximum: 1.0 };
    pub const DISABLE: Self = Self { multiplier: 0.0, maximum: 0.0 };

    pub fn merge(&mut self, other: EfficiencyModifierResult) {
        self.multiplier *= other.multiplier;
        self.maximum = self.maximum.min(other.maximum);
    }

    #[must_use]
    pub fn to_scalar(&self) -> f32 { self.multiplier.min(self.maximum) }
}

pub trait ReactionExecutor<P, D> {
    fn execute(&self, efficiency: f32, params: &mut P, data: &mut D);
}

pub trait FluidStorageSelector<P, D> {
    fn select<R>(&self, params: &P, data: &D, then: impl FnOnce(&fluid::Storage) -> R)
    -> Option<R>;

    fn select_mut<R>(
        &self,
        params: &mut P,
        data: &mut D,
        then: impl FnOnce(&mut fluid::Storage) -> R,
    ) -> Option<R>;
}

pub trait ResidentSelector<P, D> {
    fn for_each_attributes(
        &self,
        params: &P,
        data: &D,
        then: impl FnMut(&resident::Attributes, Entity),
    );

    fn for_each_attributes_mut(
        &self,
        params: &mut P,
        data: &mut D,
        then: impl FnMut(&mut resident::Attributes, Entity),
    );
}
pub fn execute_once<P, D, Input, Catalyst, Output>(
    params: &mut P,
    data: &mut D,
    inputs: &[Input],
    catalysts: &[Catalyst],
    outputs: &[Output],
    efficiency_cap: f32,
    efficiency_multiplier: f32,
) -> f32
where
    Input: EfficiencyModifier<P, D> + ReactionExecutor<P, D>,
    Catalyst: EfficiencyModifier<P, D>,
    Output: ReactionExecutor<P, D>,
{
    let mut efficiency =
        EfficiencyModifierResult { maximum: efficiency_cap, multiplier: efficiency_multiplier };

    for catalyst in catalysts {
        let modifier = EfficiencyModifier::compute_efficiency(catalyst, params, data);
        efficiency.merge(modifier);
    }
    for input in inputs {
        let modifier = EfficiencyModifier::compute_efficiency(input, params, data);
        efficiency.merge(modifier);
    }

    let efficiency = efficiency.to_scalar();
    if efficiency > 0.0 {
        for input in inputs {
            ReactionExecutor::execute(input, efficiency, params, data);
        }
        for output in outputs {
            ReactionExecutor::execute(output, efficiency, params, data);
        }
        efficiency
    } else {
        0.0
    }
}

macro_rules! define_ruleset {
    (
        [P = $P:ident, D = $D:ident]
        $(#[$input_meta:meta])* $input_vis:vis input $input:ident
        {
            $(
                $(#[$input_variant_meta:meta])*
                $input_variant:ident ($input_var_ty:ty)
            ),* $(,)?
        }
        $(#[$catalyst_meta:meta])* $catalyst_vis:vis catalyst $catalyst:ident
        {
            $(
                $(#[$catalyst_variant_meta:meta])*
                $catalyst_variant:ident ($catalyst_var_ty:ty)
            ),* $(,)?
        }
        $(#[$output_meta:meta])* $output_vis:vis output $output:ident
        {
            $(
                $(#[$output_variant_meta:meta])*
                $output_variant:ident ($output_var_ty:ty)
            ),* $(,)?
        }
    ) => {
        $(#[$input_meta])*
        $input_vis enum $input {
            $($input_variant($input_var_ty),)*
        }

        $crate::reaction::define_ruleset!(@impl EfficiencyModifier<$P, $D> for $input { $($input_variant)* });
        $crate::reaction::define_ruleset!(@impl ReactionExecutor<$P, $D> for $input { $($input_variant)* });

        $(#[$catalyst_meta])*
        $catalyst_vis enum $catalyst {
            $($catalyst_variant($catalyst_var_ty),)*
        }

        $crate::reaction::define_ruleset!(@impl EfficiencyModifier<$P, $D> for $catalyst { $($catalyst_variant)* });

        $(#[$output_meta])*
        $output_vis enum $output {
            $($output_variant($output_var_ty),)*
        }

        $crate::reaction::define_ruleset!(@impl ReactionExecutor<$P, $D> for $output { $($output_variant)* });
    };

    (@impl EfficiencyModifier<$P:ident, $D:ident> for $enum:ident { $($variant:ident)* }) => {
        impl<'pw, 'ps, 'dw, 'ds> $crate::reaction::EfficiencyModifier<$P<'pw, 'ps>, $D<'dw, 'ds>> for $enum {
            fn compute_efficiency(&self, params: &$P, data: &$D) -> $crate::reaction::EfficiencyModifierResult {
                match self {
                    $(
                        Self::$variant(inner) => {
                            $crate::reaction::EfficiencyModifier::compute_efficiency(inner, params, data)
                        }
                    )*
                }
            }
        }
    };

    (@impl ReactionExecutor<$P:ident, $D:ident> for $enum:ident { $($variant:ident)* }) => {
        impl<'pw, 'ps, 'dw, 'ds> $crate::reaction::ReactionExecutor<$P<'pw, 'ps>, $D<'dw, 'ds>> for $enum {
            fn execute(&self, efficiency: f32, params: &mut $P, data: &mut $D) {
                match self {
                    $(
                        Self::$variant(inner) => {
                            $crate::reaction::ReactionExecutor::execute(inner, efficiency, params, data)
                        }
                    )*
                }
            }
        }
    };
}

pub(crate) use define_ruleset;
