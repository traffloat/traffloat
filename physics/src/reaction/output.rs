use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::reaction::{FluidStorageSelector, ReactionExecutor, ResidentSelector};
use crate::{fluid, resident};

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Fluid<S> {
    pub selector: S,
    /// The type of fluid to produce.
    pub ty:       fluid::TypeId,
    /// Maximum number of moles to produce per timestep at maximum efficiency.
    pub max_rate: fluid::Moles,
}

impl<S, P, D> ReactionExecutor<P, D> for Fluid<S>
where
    S: FluidStorageSelector<P, D>,
{
    fn execute(&self, efficiency: f32, params: &mut P, data: &mut D) {
        self.selector.select_mut(params, data, |storage| {
            let typed = storage.get_type_mut(self.ty);
            typed.moles += fluid::Moles(efficiency * self.max_rate.0);
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Heat<S> {
    pub selector: S,
    /// Maximum amount of heat to produce per timestep at maximum efficiency.
    pub max_rate: fluid::Energy,
}

impl<S, P, D> ReactionExecutor<P, D> for Heat<S>
where
    S: FluidStorageSelector<P, D>,
{
    fn execute(&self, efficiency: f32, params: &mut P, data: &mut D) {
        self.selector.select_mut(params, data, |storage| {
            storage.heat += fluid::Energy(efficiency * self.max_rate.0);
        });
    }
}

/// Modifies an attribute of residents in a slot.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ResidentAttr<S> {
    pub selector: S,
    /// The type of attribute to modify.
    pub attr:     resident::attr::TypeId,
    /// The linear change to apply to the attribute value of each resident per timestep.
    pub delta:    f32,
    /// Minimum value to clamp the attribute to after applying the delta.
    pub min:      Option<f32>,
    /// Maximum value to clamp the attribute to after applying the delta.
    pub max:      Option<f32>,
}

impl<S, P, D> ReactionExecutor<P, D> for ResidentAttr<S>
where
    S: ResidentSelector<P, D>,
{
    fn execute(&self, efficiency: f32, params: &mut P, data: &mut D) {
        self.selector.for_each_attributes_mut(params, data, |attrs, _| {
            let attr = attrs.get_mut(self.attr);
            *attr += self.delta * efficiency;
            if let Some(min) = self.min {
                *attr = attr.max(min);
            }
            if let Some(max) = self.max {
                *attr = attr.min(max);
            }
        });
    }
}
