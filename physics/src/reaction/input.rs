use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::reaction::{
    CargoStorageSelector, ContinuousReactionExecutor, EfficiencyModifier, EfficiencyModifierResult,
    FluidStorageSelector, GranularReactionExecutor, Threshold,
};
use crate::{cargo, fluid};

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Fluid<S> {
    pub selector:       S,
    /// The type of fluid to take.
    pub ty:             fluid::TypeId,
    /// Maximum number of moles to take per timestep at maximum efficiency.
    pub max_rate:       fluid::Moles,
    /// How fluid concentration affects the efficiency of the reaction.
    ///
    /// X is the molar concentration.
    pub conc_threshold: Threshold,
}

impl<S, P, D> EfficiencyModifier<P, D> for Fluid<S>
where
    S: FluidStorageSelector<P, D>,
{
    fn compute_efficiency(&self, params: &P, data: &D) -> EfficiencyModifierResult {
        self.selector
            .select(params, data, |storage| {
                let typed = storage.get_type(self.ty);
                let mut out = self.conc_threshold.lerp(typed.molar_conc);
                if typed.moles < self.max_rate {
                    out.maximum = out.maximum.min(typed.moles.0 / self.max_rate.0);
                }
                out
            })
            .unwrap_or(EfficiencyModifierResult::DISABLE)
    }
}

impl<S, P, D> ContinuousReactionExecutor<P, D> for Fluid<S>
where
    S: FluidStorageSelector<P, D>,
{
    fn execute_continuous(&self, efficiency: f32, params: &mut P, data: &mut D) {
        self.selector.select_mut(params, data, |storage| {
            let typed = storage.get_type_mut(self.ty);
            // The min branch is mathematically impossible since the efficiency would have reduced accordingly,
            // but we still include it to avoid floating point errors leading to negative values,
            // which could in turn result in a lot of unexpected behavior.
            typed.moles -= fluid::Moles((efficiency * self.max_rate.0).min(typed.moles.0));
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Heat<S> {
    pub selector:       S,
    /// Maximum amount of heat to take per timestep at maximum efficiency.
    pub max_rate:       fluid::Energy,
    /// How temperature affects the efficiency of the reaction.
    ///
    /// X is the temperature in kelvins.
    pub temp_threshold: Threshold,
}

impl<S, P, D> EfficiencyModifier<P, D> for Heat<S>
where
    S: FluidStorageSelector<P, D>,
{
    fn compute_efficiency(&self, params: &P, data: &D) -> EfficiencyModifierResult {
        self.selector
            .select(params, data, |storage| {
                let mut out = self.temp_threshold.lerp(storage.temperature);
                if storage.heat < self.max_rate {
                    out.maximum = out.maximum.min(storage.heat.0 / self.max_rate.0);
                }
                out
            })
            .unwrap_or(EfficiencyModifierResult::DISABLE)
    }
}

impl<S, P, D> ContinuousReactionExecutor<P, D> for Heat<S>
where
    S: FluidStorageSelector<P, D>,
{
    fn execute_continuous(&self, efficiency: f32, params: &mut P, data: &mut D) {
        self.selector.select_mut(params, data, |storage| {
            // The min branch is mathematically impossible, see comment in Fluid::execute.
            let heat_to_take = fluid::Energy((efficiency * self.max_rate.0).min(storage.heat.0));
            storage.heat -= heat_to_take;
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Cargo<S> {
    pub selector: S,
    pub ty:       cargo::TypeId,
    pub granules: cargo::Granules,
}

impl<S, P, D> EfficiencyModifier<P, D> for Cargo<S>
where
    S: CargoStorageSelector<P, D>,
{
    fn compute_efficiency(&self, params: &P, data: &D) -> EfficiencyModifierResult {
        self.selector
            .select(params, data, |storage| {
                let has_granules = storage.get_type(self.ty).map_or_default(|t| t.granules);
                let mut out = EfficiencyModifierResult::default();
                if has_granules < self.granules {
                    out.maximum = out.maximum.min(has_granules.div_f32(self.granules));
                }
                out
            })
            .unwrap_or(EfficiencyModifierResult::DISABLE)
    }
}

impl<S, P, D> GranularReactionExecutor<P, D> for Cargo<S>
where
    S: CargoStorageSelector<P, D>,
{
    fn execute_granule(&self, params: &mut P, data: &mut D, multiplier: u32) {
        self.selector.select_mut(params, data, |storage| {
            if let Some(typed) = storage.get_type_mut(self.ty) {
                // The min branch is mathematically impossible since the efficiency would have reduced accordingly,
                // but we still include it just to avoid panicking.
                typed.granules -= (self.granules * multiplier).min(typed.granules);
            }
        });
    }
}
