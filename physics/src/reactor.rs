use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::{IntoScheduleConfigs, SystemSet};
use bevy::ecs::system::{Local, Query, Res, SystemParam};
use enum_dispatch::enum_dispatch;

use crate::fluid;
use crate::util::{QueryExt, SliceGet};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.init_resource::<Types>();
        app.init_resource::<Conf>();
        app.add_systems(app::FixedUpdate, execute_system.in_set(ExecuteSystemSet));
    }
}

#[derive(Resource)]
pub struct Conf {
    pub execution_timestep: u32,
}

impl Default for Conf {
    fn default() -> Self { Self { execution_timestep: 16 } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct ExecuteSystemSet;

#[derive(SystemParam)]
struct ExecuteSystemParams<'w, 's> {
    fluid_storage: Query<'w, 's, &'static mut fluid::Storage>,
}

// In the future we may optimize this module to use dynamic components and systems
// for better performance and parallelism,
// but for now we will keep it simple and use a single component for reactors
// and iterate over all reactors in series.
fn execute_system(
    conf: Res<Conf>,
    mut next_step: Local<u32>,
    reactor_query: Query<&Facility>,
    types: Res<Types>,
    mut params: ExecuteSystemParams,
) {
    *next_step += 1;
    *next_step %= conf.execution_timestep;
    if *next_step != 0 {
        return;
    }

    for reactor in reactor_query {
        let def = types.get(reactor.id);
        execute_once(reactor, def, &mut params);
    }
}

fn execute_once(reactor: &Facility, def: &TypeDef, params: &mut ExecuteSystemParams) {
    let mut efficiency =
        EfficiencyModifierResult { maximum: reactor.efficiency_cap, multiplier: 1.0 };

    for catalyst in &def.catalysts {
        efficiency.merge(catalyst.compute_efficiency(params, reactor));
    }

    for input in &def.inputs {
        efficiency.merge(input.compute_efficiency(params, reactor));
    }

    let efficiency = efficiency.to_scalar();
    if efficiency > 0.0 {
        for input in &def.inputs {
            input.execute(efficiency, params, reactor);
        }

        for output in &def.outputs {
            output.execute(efficiency, params, reactor);
        }
    }
}

#[derive(Component)]
pub struct Facility {
    pub id:             TypeId,
    pub efficiency_cap: f32,
    pub refs:           Refs,
}

pub struct Refs {
    pub fluid_storage: Vec<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(pub u32);

#[derive(Resource, Default)]
pub struct Types {
    pub types: Vec<TypeDef>,
}

impl Types {
    #[must_use]
    pub fn get(&self, id: TypeId) -> &TypeDef {
        self.types.get(id.0 as usize).expect("got invalid reactor type reference")
    }
}

/// A component on facilities.
pub struct TypeDef {
    pub inputs:    Vec<Input>,
    pub outputs:   Vec<Output>,
    pub catalysts: Vec<Catalyst>,
}

/// A reference to an entry in [`Refs::fluid_storage`].
pub struct FluidStorageRef(pub u32);

#[enum_dispatch]
trait EfficiencyModifier {
    fn compute_efficiency(
        &self,
        params: &ExecuteSystemParams,
        reactor: &Facility,
    ) -> EfficiencyModifierResult;
}

#[enum_dispatch]
trait ReactionExecutor {
    fn execute(&self, efficiency: f32, params: &mut ExecuteSystemParams, reactor: &Facility);
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
    pub const INVALID: Self = Self { multiplier: 0.0, maximum: 0.0 };

    pub fn merge(&mut self, other: EfficiencyModifierResult) {
        self.multiplier *= other.multiplier;
        self.maximum = self.maximum.min(other.maximum);
    }

    #[must_use]
    pub fn to_scalar(&self) -> f32 { self.multiplier.min(self.maximum) }
}

#[enum_dispatch(EfficiencyModifier)]
#[enum_dispatch(ReactionExecutor)]
pub enum Input {
    /// Removes fluid from a storage.
    Fluid(FluidInput),
    /// Removes heat from a storage.
    ///
    /// For reactors that consume coldness instead,
    /// they should use a catalyst and an output.
    Heat(HeatInput),
}

pub struct FluidInput {
    /// The storage entity to take fluid from.
    pub storage:        FluidStorageRef,
    /// The type of fluid to take.
    pub ty:             fluid::TypeId,
    /// Maximum number of moles to take per timestep when reactor is at maximum efficiency.
    pub max_rate:       fluid::Moles,
    /// How fluid concentration affects the efficiency of the reactor.
    pub conc_threshold: Threshold,
}

impl EfficiencyModifier for FluidInput {
    fn compute_efficiency(
        &self,
        params: &ExecuteSystemParams,
        reactor: &Facility,
    ) -> EfficiencyModifierResult {
        let Some(&storage_entity) = reactor.refs.fluid_storage.log_get(self.storage.0 as usize)
        else {
            return EfficiencyModifierResult::INVALID;
        };
        let Some(storage) = params.fluid_storage.log_get(storage_entity) else {
            return EfficiencyModifierResult::INVALID;
        };
        let typed = storage.get_type(self.ty);
        let mut out = self.conc_threshold.lerp(typed.molar_conc);
        if typed.moles < self.max_rate {
            out.maximum = out.maximum.min(typed.moles.0 / self.max_rate.0);
        }
        out
    }
}

impl ReactionExecutor for FluidInput {
    fn execute(&self, efficiency: f32, params: &mut ExecuteSystemParams, reactor: &Facility) {
        let Some(&storage_entity) = reactor.refs.fluid_storage.log_get(self.storage.0 as usize)
        else {
            return;
        };
        let Some(mut storage) = params.fluid_storage.log_get_mut(storage_entity) else { return };
        let typed = storage.get_type_mut(self.ty);
        // The min branch is mathematically impossible since the efficiency would have reduced accordingly,
        // but we still include it to avoid floating point errors leading to negative values,
        // which could in turn result in a lot of unexpected behavior.
        typed.moles -= fluid::Moles((efficiency * self.max_rate.0).min(typed.moles.0));
    }
}

pub struct HeatInput {
    /// The storage entity to take heat from.
    pub storage:        FluidStorageRef,
    /// Maximum amount of heat to take per timestep when reactor is at maximum efficiency.
    pub max_rate:       fluid::Energy,
    /// How temperature affects the efficiency of the reactor.
    pub temp_threshold: Threshold,
}

impl EfficiencyModifier for HeatInput {
    fn compute_efficiency(
        &self,
        params: &ExecuteSystemParams,
        reactor: &Facility,
    ) -> EfficiencyModifierResult {
        let Some(&storage_entity) = reactor.refs.fluid_storage.log_get(self.storage.0 as usize)
        else {
            return EfficiencyModifierResult::INVALID;
        };
        let Some(storage) = params.fluid_storage.log_get(storage_entity) else {
            return EfficiencyModifierResult::INVALID;
        };
        let mut out = self.temp_threshold.lerp(storage.temperature);
        if storage.heat < self.max_rate {
            out.maximum = out.maximum.min(storage.heat.0 / self.max_rate.0);
        }
        out
    }
}

impl ReactionExecutor for HeatInput {
    fn execute(&self, efficiency: f32, params: &mut ExecuteSystemParams, reactor: &Facility) {
        let Some(&storage_entity) = reactor.refs.fluid_storage.log_get(self.storage.0 as usize)
        else {
            return;
        };
        let Some(mut storage) = params.fluid_storage.log_get_mut(storage_entity) else { return };
        // The min branch is mathematically impossible, see comment in FluidInput::execute.
        let heat_to_take = fluid::Energy((efficiency * self.max_rate.0).min(storage.heat.0));
        storage.heat -= heat_to_take;
    }
}

#[enum_dispatch(ReactionExecutor)]
pub enum Output {
    Fluid(FluidOutput),
    Temperature(TemperatureOutput),
}

pub struct FluidOutput {
    /// The storage entity to put fluid into.
    pub storage:  FluidStorageRef,
    /// The type of fluid to produce.
    pub ty:       fluid::TypeId,
    /// Maximum number of moles to produce per timestep when reactor is at maximum efficiency.
    pub max_rate: fluid::Moles,
}

impl ReactionExecutor for FluidOutput {
    fn execute(&self, efficiency: f32, params: &mut ExecuteSystemParams, reactor: &Facility) {
        let Some(&storage_entity) = reactor.refs.fluid_storage.log_get(self.storage.0 as usize)
        else {
            return;
        };
        let Some(mut storage) = params.fluid_storage.log_get_mut(storage_entity) else { return };
        let typed = storage.get_type_mut(self.ty);
        typed.moles += fluid::Moles(efficiency * self.max_rate.0);
    }
}

pub struct TemperatureOutput {
    /// The storage entity to put heat into.
    pub storage:  FluidStorageRef,
    /// Maximum amount of heat to produce per timestep when reactor is at maximum efficiency.
    pub max_rate: fluid::Energy,
}

impl ReactionExecutor for TemperatureOutput {
    fn execute(&self, efficiency: f32, params: &mut ExecuteSystemParams, reactor: &Facility) {
        let Some(&storage_entity) = reactor.refs.fluid_storage.log_get(self.storage.0 as usize)
        else {
            return;
        };
        let Some(mut storage) = params.fluid_storage.log_get_mut(storage_entity) else { return };
        storage.heat += fluid::Energy(efficiency * self.max_rate.0);
    }
}

#[enum_dispatch(EfficiencyModifier)]
pub enum Catalyst {
    Fluid(FluidCatalyst),
    Pressure(PressureCatalyst),
    Temperature(TemperatureCatalyst),
}

pub struct FluidCatalyst {
    /// The storage entity to check.
    pub storage:        FluidStorageRef,
    /// The type of fluid to take.
    pub ty:             fluid::TypeId,
    /// How fluid concentration affects the efficiency of the reactor.
    pub conc_threshold: Threshold,
}

impl EfficiencyModifier for FluidCatalyst {
    fn compute_efficiency(
        &self,
        params: &ExecuteSystemParams,
        reactor: &Facility,
    ) -> EfficiencyModifierResult {
        let Some(&storage_entity) = reactor.refs.fluid_storage.log_get(self.storage.0 as usize)
        else {
            return EfficiencyModifierResult::default();
        };
        let Some(storage) = params.fluid_storage.log_get(storage_entity) else {
            return EfficiencyModifierResult::default();
        };
        let typed = storage.get_type(self.ty);
        self.conc_threshold.lerp(typed.molar_conc)
    }
}

pub struct PressureCatalyst {
    /// The fluid storage entity to check.
    pub storage:            FluidStorageRef,
    /// How pressure affects the efficiency of the reactor.
    pub pressure_threshold: Threshold,
}

impl EfficiencyModifier for PressureCatalyst {
    fn compute_efficiency(
        &self,
        params: &ExecuteSystemParams,
        reactor: &Facility,
    ) -> EfficiencyModifierResult {
        let Some(&storage_entity) = reactor.refs.fluid_storage.log_get(self.storage.0 as usize)
        else {
            return EfficiencyModifierResult::default();
        };
        let Some(storage) = params.fluid_storage.log_get(storage_entity) else {
            return EfficiencyModifierResult::default();
        };
        self.pressure_threshold.lerp(storage.pressure)
    }
}

pub struct TemperatureCatalyst {
    /// The fluid storage entity to chcek.
    pub storage:        FluidStorageRef,
    /// How temperature affects the efficiency of the reactor.
    pub temp_threshold: Threshold,
}

impl EfficiencyModifier for TemperatureCatalyst {
    fn compute_efficiency(
        &self,
        params: &ExecuteSystemParams,
        reactor: &Facility,
    ) -> EfficiencyModifierResult {
        let Some(&storage_entity) = reactor.refs.fluid_storage.log_get(self.storage.0 as usize)
        else {
            return EfficiencyModifierResult::default();
        };
        let Some(storage) = params.fluid_storage.log_get(storage_entity) else {
            return EfficiencyModifierResult::default();
        };
        self.temp_threshold.lerp(storage.temperature)
    }
}

pub struct Threshold {
    /// Start X-coordinate of the interpolation curve.
    pub min_input:                 f32,
    /// End X-coordinate of the interpolation curve.
    pub max_input:                 f32,
    /// Start Y-coordinate of the interpolation curve.
    pub min_efficiency_multiplier: f32,
    /// End Y-coordinate of the interpolation curve.
    pub max_efficiency_multiplier: f32,
    /// The interpolation curve to use between the min and max input.
    pub curve:                     Curve,
    /// How this threshold modifies the efficiency of the reactor.
    pub modifier_type:             ThresholdModifierType,
}

impl Threshold {
    #[must_use]
    pub fn lerp(&self, input: f32) -> EfficiencyModifierResult {
        let clamped_input = input.clamp(self.min_input, self.max_input);
        let t = (clamped_input - self.min_input) / (self.max_input - self.min_input);
        let out = match self.curve {
            Curve::Linear => {
                self.min_efficiency_multiplier
                    + t * (self.max_efficiency_multiplier - self.min_efficiency_multiplier)
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

pub enum ThresholdModifierType {
    /// This threshold multiplies the efficiency by the output of the curve.
    Multiplier,
    /// This threshold restricts efficiency from exceeding the output of the curve.
    Maximum,
}

pub enum Curve {
    Linear,
}
