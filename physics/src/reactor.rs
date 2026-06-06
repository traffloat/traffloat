use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::{IntoScheduleConfigs, SystemSet};
use bevy::ecs::system::{Local, Query, Res, SystemParam};
use bevy::math::FloatExt;
use bevy::reflect::Reflect;
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

/// Component on facilities.
#[derive(Component)]
pub struct Facility {
    pub id:             TypeId,
    /// The maximum efficiency as configured by the player.
    pub efficiency_cap: f32,
    pub ports:          Ports,
}

pub struct Ports {
    pub fluid_storages: Vec<Option<Entity>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
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

    pub fn push(&mut self, def: TypeDef) -> TypeId {
        let id = u32::try_from(self.types.len()).expect("too many reactor types");
        self.types.push(def);
        TypeId(id)
    }
}

/// A component on facilities.
pub struct TypeDef {
    pub inputs:    Vec<Input>,
    pub outputs:   Vec<Output>,
    pub catalysts: Vec<Catalyst>,
}

/// A reference to an entry in [`Refs::fluid_storage`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        let Some(&Some(storage_entity)) = reactor.ports.fluid_storages.get(self.storage.0 as usize)
        else {
            tracing::warn!(
                "reactor port {:?} is used as an input and must not be nil",
                self.storage
            );
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
        let Some(&Some(storage_entity)) = reactor.ports.fluid_storages.get(self.storage.0 as usize)
        else {
            tracing::warn!(
                "reactor port {:?} is used as an input and must not be nil",
                self.storage
            );
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
        let Some(&Some(storage_entity)) = reactor.ports.fluid_storages.get(self.storage.0 as usize)
        else {
            tracing::warn!(
                "reactor port {:?} is used as an input and must not be nil",
                self.storage
            );
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
        let Some(&Some(storage_entity)) = reactor.ports.fluid_storages.get(self.storage.0 as usize)
        else {
            tracing::warn!(
                "reactor port {:?} is used as an input and must not be nil",
                self.storage
            );
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
        let Some(&Some(storage_entity)) = reactor.ports.fluid_storages.get(self.storage.0 as usize)
        else {
            tracing::warn!(
                "reactor port {:?} is used as an output and must not be nil",
                self.storage
            );
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
        let Some(&Some(storage_entity)) = reactor.ports.fluid_storages.get(self.storage.0 as usize)
        else {
            tracing::warn!(
                "reactor port {:?} is used as an output and must not be nil",
                self.storage
            );
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
        let Some(&maybe_storage) = reactor.ports.fluid_storages.log_get(self.storage.0 as usize)
        else {
            return EfficiencyModifierResult::default();
        };
        let storage = if let Some(storage_entity) = maybe_storage
            && let Some(storage) = params.fluid_storage.log_get(storage_entity)
        {
            storage
        } else {
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
        let Some(&maybe_storage) = reactor.ports.fluid_storages.log_get(self.storage.0 as usize)
        else {
            return EfficiencyModifierResult::default();
        };
        let storage = if let Some(storage_entity) = maybe_storage
            && let Some(storage) = params.fluid_storage.log_get(storage_entity)
        {
            storage
        } else {
            return EfficiencyModifierResult::default();
        };
        self.pressure_threshold.lerp(storage.pressure)
    }
}

pub struct TemperatureCatalyst {
    /// The fluid storage entity to check.
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
        let Some(&maybe_storage) = reactor.ports.fluid_storages.log_get(self.storage.0 as usize)
        else {
            return EfficiencyModifierResult::default();
        };
        let storage = if let Some(storage_entity) = maybe_storage
            && let Some(storage) = params.fluid_storage.log_get(storage_entity)
        {
            storage
        } else {
            return EfficiencyModifierResult::default();
        };
        self.temp_threshold.lerp(storage.temperature)
    }
}

pub struct Threshold {
    /// The interpolation curve to use between the min and max input.
    pub curve:         Curve,
    /// How this threshold modifies the efficiency of the reactor.
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

pub enum ThresholdModifierType {
    /// This threshold multiplies the efficiency by the output of the curve.
    Multiplier,
    /// This threshold restricts efficiency from exceeding the output of the curve.
    Maximum,
}

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
