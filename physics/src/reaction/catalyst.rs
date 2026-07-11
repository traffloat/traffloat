use bevy::ecs::entity::Entity;
use bevy::ecs::world::Mut;
use bevy::reflect::Reflect;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use crate::reaction::{
    Aggregator, EfficiencyModifier, EfficiencyModifierResult, FluidStorageSelector,
    ReactionExecutor, ResidentSelector, Threshold,
};
use crate::{fluid, resident};

/// Concentration of a fluid in a connected fluid storage.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Fluid<S> {
    pub selector:       S,
    /// The type of fluid to take.
    pub ty:             fluid::TypeId,
    /// How fluid concentration affects the efficiency of the reactor.
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
                self.conc_threshold.lerp(typed.molar_conc)
            })
            .unwrap_or(EfficiencyModifierResult::default())
    }
}

/// Pressure in a connected fluid storage.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Pressure<S> {
    pub selector:           S,
    /// How pressure affects the efficiency of the reactor.
    pub pressure_threshold: Threshold,
}

impl<S, P, D> EfficiencyModifier<P, D> for Pressure<S>
where
    S: FluidStorageSelector<P, D>,
{
    fn compute_efficiency(&self, params: &P, data: &D) -> EfficiencyModifierResult {
        self.selector
            .select(params, data, |storage| self.pressure_threshold.lerp(storage.pressure))
            .unwrap_or(EfficiencyModifierResult::default())
    }
}

/// Temperature in a connected fluid storage.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Temperature<S> {
    pub selector:       S,
    /// How temperature affects the efficiency of the reactor.
    pub temp_threshold: Threshold,
}

impl<S, P, D> EfficiencyModifier<P, D> for Temperature<S>
where
    S: FluidStorageSelector<P, D>,
{
    fn compute_efficiency(&self, params: &P, data: &D) -> EfficiencyModifierResult {
        self.selector
            .select(params, data, |storage| self.temp_threshold.lerp(storage.temperature))
            .unwrap_or(EfficiencyModifierResult::default())
    }
}

/// Attribute of residents interacting with the reactor.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ResidentAttr<S> {
    /// The interaction slot index of the residents that can catalyst the reactor.
    pub selector:   S,
    /// The relevant attribute.
    pub attr:       resident::attr::TypeId,
    /// How multiple interacting residents are aggregated into a single value.
    pub aggregator: Aggregator,
    /// How aggregated attribute value affects the efficiency of the reactor.
    pub threshold:  Threshold,
}

impl<S, P, D> EfficiencyModifier<P, D> for ResidentAttr<S>
where
    S: ResidentSelector<P, D>,
{
    fn compute_efficiency(&self, params: &P, data: &D) -> EfficiencyModifierResult {
        let mut result = self.aggregator.initial();
        self.selector.for_each_attributes(params, data, |attrs, _| {
            let attr = attrs.get(self.attr);
            self.aggregator.reduce(&mut result, attr);
        });
        self.threshold.lerp(result)
    }
}
