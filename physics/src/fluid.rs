use std::{cmp, iter, mem, ops};

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::query::{QueryData, With};
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::{IntoScheduleConfigs, SystemSet};
use bevy::ecs::system::{Command, Commands, EntityCommand, Local, Query, Res, SystemParam};
use bevy::ecs::world::{EntityWorldMut, World};
use bevy::reflect::Reflect;
use traffloat_proto::proto;

use crate::util::{AlphaBeta, MergeSortedItem, QueryExt, merge_sorted};
use crate::view;

#[cfg(test)]
mod tests;

/// A constant equivalent to the ideal gas constant, used for pressure calculation.
pub const PRESSURE_COEFFICIENT: f32 = 1.0 / 128.0;

/// A constant to adjust the base diffusion rate.
pub const DIFFUSION_COEFFICIENT: f32 = 1.0 / 8192.0;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Types>();
        app.register_type::<TypeDef>();
        app.register_type::<Storage>();
        app.register_type::<Edge>();
        app.register_type::<AlphaOfEdgeList>();
        app.register_type::<EdgeAlpha>();
        app.register_type::<BetaOfEdgeList>();
        app.register_type::<EdgeBeta>();
        app.register_type::<ViewerSynced>();

        app.init_resource::<Conf>();
        app.init_resource::<Types>();

        app.add_systems(app::FixedUpdate, transfer_system.in_set(TransferSystemSet));
        app.add_systems(app::Update, sync_types_to_viewers_system);
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransferSystemSet;

#[derive(Resource)]
pub struct Conf {
    pub transfer_timestep: u32,
}

impl Default for Conf {
    fn default() -> Self { Self { transfer_timestep: 16 } }
}

/// Amount of fluid substance,
/// in an arbitrary unit equivalent to moles.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    PartialOrd,
    derive_more::Add,
    derive_more::AddAssign,
    derive_more::Sub,
    derive_more::SubAssign,
    Reflect,
)]
pub struct Moles(pub f32);

impl ops::Mul<f32> for Moles {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self { Self(self.0 * rhs) }
}

/// Amount of heat energy,
/// in an arbitrary unit equivalent to joules.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    PartialOrd,
    derive_more::Add,
    derive_more::AddAssign,
    derive_more::Sub,
    derive_more::SubAssign,
    Reflect,
)]
pub struct Energy(pub f32);

impl ops::Mul<f32> for Energy {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self { Self(self.0 * rhs) }
}

/// Identifies a fluid type, indexes [`Types::types`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub struct TypeId(pub u32);

#[derive(Resource, Reflect, Default)]
pub struct Types {
    pub types: Vec<TypeDef>,
}

impl Types {
    #[must_use]
    pub fn get(&self, ty: TypeId) -> &TypeDef {
        self.types.get(ty.0 as usize).expect("invalid fluid type reference created")
    }

    pub fn push(&mut self, type_def: TypeDef) -> TypeId {
        let id = u32::try_from(self.types.len()).expect("too many fluid types");
        self.types.push(type_def);
        TypeId(id)
    }
}

pub struct AddTypeCommand {
    pub type_def: TypeDef,
}

impl Command for AddTypeCommand {
    fn apply(self, world: &mut World) {
        let mut types = world.resource_mut::<Types>();
        types.push(self.type_def);
        let num_types = types.types.len();

        for mut storage in world.query::<&mut Storage>().iter_mut(world) {
            let new_typed =
                (0..num_types).map(|i| storage.types.get(i).copied().unwrap_or_default()).collect();
            storage.types = new_typed;
        }

        for mut edge in world.query::<&mut Edge>().iter_mut(world) {
            let new_typed = (0..num_types)
                .map(|i| edge.last_typed_transfer.get(i).cloned().unwrap_or_default())
                .collect();
            edge.last_typed_transfer = new_typed;
        }
    }
}

#[derive(Debug, Reflect)]
pub struct TypeDef {
    pub name:                 String,
    /// Heat energy per mole.
    pub molar_heat_capacity:  f32,
    /// Mass per mole.
    pub molar_density:        f32,
    /// Multiplier to advection rate.
    ///
    /// Must not exceed 1.0.
    pub advective_fluidity:   f32,
    /// Multiplier to diffusion rate.
    ///
    /// Must not exceed 1.0.
    pub diffusive_fluidity:   f32,
    /// Multiplier to heat transfer through conduction.
    pub thermal_conductivity: f32,
    /// Base multiplier for transmitted color.
    pub optical_extinction:   [f32; 3],
    // /// Base multiplier for emitted color.
    // pub optical_emission:  [f32; 3],
}

#[derive(Component, Reflect, Debug, Clone)]
pub struct Storage {
    /// Volume provided by the storage.
    ///
    /// This may change over time subject to displacement,
    /// e.g. dumping cargo into a storage.
    ///
    /// May be mutated by modules defining a storage.
    pub volume: f32,
    /// Length of the storage to compute light absorption rate.
    pub length: f32,

    /// Heat energy in the storage.
    pub heat:  Energy,
    /// Must always be sorted by type.
    // TODO benchmark possible alternative representations:
    // 1. use smallvec
    // 2. use dynamic components for each type
    pub types: Vec<TypedStorage>,

    // derived quantities
    /// Force per unit area exerted by the mixture.
    pub pressure:    f32,
    /// Absolute temperature.
    pub temperature: f32,
    /// Mass of fluid in this storage, used for force calculation in other modules.
    pub mass:        f32,
    /// Total moles of fluid in this storage.
    pub moles:       Moles,

    pub optical_extinction: [f32; 3],
    // pub optical_emission: [f32; 3],
    /// RGBA color of the container.
    pub rgba:               [f32; 4],
}

impl Storage {
    #[must_use]
    pub fn vacuum(num_types: usize, volume: f32, length: f32) -> Self {
        Self {
            volume,
            length,
            heat: Energy(0.0),
            types: (0..num_types).map(|_| TypedStorage::default()).collect(),
            pressure: 0.0,
            temperature: 0.0,
            mass: 0.0,
            moles: Moles(0.0),
            optical_extinction: [0.0; 3],
            // optical_emission: [0.0; 3],
            rgba: [0.0; 4],
        }
    }

    #[must_use]
    pub fn with_heat(mut self, heat: Energy) -> Self {
        self.heat = heat;
        self
    }

    pub fn set_heat(&mut self, heat: Energy) { self.heat = heat; }

    #[must_use]
    pub fn with_fluid(mut self, ty: TypeId, moles: f32) -> Self {
        self.set_fluid(ty, Moles(moles));
        self
    }

    pub fn set_fluid(&mut self, ty: TypeId, moles: Moles) {
        // for performance reasons,
        // we are not going to update proportion and molar conc until the next tick.
        // This is expected to have negligible impact
        // since it only affects flow rate multiplier computation for one tick.

        self.get_type_mut(ty).moles = moles;
    }

    /// Computes total heat capacity based on derived quantities.
    #[must_use]
    pub fn derived_total_heat_capacity(&self) -> f32 {
        if self.heat.0 == 0.0 || self.temperature == 0.0 {
            return 0.0;
        }
        self.heat.0 / self.temperature
    }

    #[must_use]
    pub fn get_type(&self, ty: TypeId) -> &TypedStorage {
        self.types
            .get(ty.0 as usize)
            .expect("all fluid storages must be resized after adding fluid storages")
    }

    #[must_use]
    pub fn get_type_mut(&mut self, ty: TypeId) -> &mut TypedStorage {
        self.types
            .get_mut(ty.0 as usize)
            .expect("all fluid storages must be resized after adding fluid storages")
    }

    pub fn types(&self) -> impl Iterator<Item = (TypeId, &TypedStorage)> {
        self.types
            .iter()
            .enumerate()
            .map(|(i, t)| (TypeId(u32::try_from(i).expect("too many types in storage")), t))
    }

    pub fn types_mut(&mut self) -> impl Iterator<Item = (TypeId, &mut TypedStorage)> {
        self.types
            .iter_mut()
            .enumerate()
            .map(|(i, t)| (TypeId(u32::try_from(i).expect("too many types in storage")), t))
    }

    pub fn to_proto(&self) -> proto::FluidStorageFull {
        proto::FluidStorageFull {
            volume:      self.volume,
            pressure:    self.pressure,
            temperature: self.temperature,
            types:       self.types.iter().map(|typed| typed.moles.0).collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Reflect)]
pub struct TypedStorage {
    /// Amount of this type in this storage, in moles.
    pub moles: Moles,

    // derived quantities
    /// Proportion of volume in this storage occupied by this type, between 0 and 1.
    pub proportion: f32,
    /// Moles per unit volume of this type in this storage.
    pub molar_conc: f32,
}

pub struct AddStorageCommand {
    pub volume:         f32,
    pub optical_length: f32,
}

impl EntityCommand for AddStorageCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        let num_types = entity.world().resource::<Types>().types.len();
        entity.insert(Storage::vacuum(num_types, self.volume, self.optical_length));
    }
}

pub struct SetTemperatureCommand {
    pub temperature: f32,
}

impl EntityCommand for SetTemperatureCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        let storage = try_log!(entity.get::<Storage>(), expect "SetTemperatureCommand must be applied on fluid storage entities" or return);
        let types = entity.world().resource::<Types>();
        let heat = self.temperature
            * storage
                .types()
                .map(|(type_id, typed)| {
                    let type_def = types.get(type_id);
                    typed.moles.0 * type_def.molar_heat_capacity
                })
                .sum::<f32>();

        let mut storage = entity.get_mut::<Storage>().expect("checked at function start");
        storage.heat = Energy(heat);
    }
}

/// The connection between two storages, through which fluid can transfer.
#[derive(Component, Reflect, Debug, Clone)]
pub struct Edge {
    /// Reciprocal of resistance to transfer.
    ///
    /// The reciprocal is proportional to flow rate.
    /// Physically resembles the reciprocal of the length of a pipe.
    pub resistance_recip: f32,
    /// Cross-sectional area of the connection.
    ///
    /// Directly proportional to flow rate and heat conduction rate.
    pub area:             f32,

    /// Additional force per unit area from alpha to beta, e.g. from a pump or valve.
    pub force_atob: f32,

    // Temporary states.
    /// Heat transferred from alpha to beta in the last transfer step.
    last_heat:           Energy,
    /// Must always be sorted by type.
    last_typed_transfer: Vec<TypedTransfer>,
}

impl Edge {
    #[must_use]
    pub fn new(num_types: usize, resistance_recip: f32, area: f32) -> Self {
        Self {
            resistance_recip,
            area,
            force_atob: 0.0,
            last_heat: Energy(0.0),
            last_typed_transfer: (0..num_types).map(|_| TypedTransfer::default()).collect(),
        }
    }

    #[must_use]
    pub fn with_force_atob(mut self, force: f32) -> Self {
        self.force_atob = force;
        self
    }

    #[must_use]
    pub fn with_force_btoa(mut self, force: f32) -> Self {
        self.force_atob = -force;
        self
    }
}

#[derive(Debug, Clone, Default, Reflect)]
struct TypedTransfer {
    atob_transfer: Moles,
}

pub struct AddEdgeCommand {
    pub resistance_recip: f32,
    pub area:             f32,

    pub alpha: Entity,
    pub beta:  Entity,
}

impl EntityCommand for AddEdgeCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        let num_types = entity.world().resource::<Types>().types.len();
        entity.insert((
            Edge::new(num_types, self.resistance_recip, self.area),
            EdgeAlpha(self.alpha),
            EdgeBeta(self.beta),
        ));
    }
}

#[derive(Component, Reflect)]
#[relationship(relationship_target = AlphaOfEdgeList)]
pub struct EdgeAlpha(pub Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship = EdgeAlpha, linked_spawn)]
pub struct AlphaOfEdgeList(Vec<Entity>);

#[derive(Component, Reflect)]
#[relationship(relationship_target = BetaOfEdgeList)]
pub struct EdgeBeta(pub Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship = EdgeBeta, linked_spawn)]
pub struct BetaOfEdgeList(Vec<Entity>);

#[derive(QueryData)]
#[query_data(mutable)]
struct TransferStorageData {
    storage:     &'static mut Storage,
    edges_alpha: Option<&'static AlphaOfEdgeList>,
    edges_beta:  Option<&'static BetaOfEdgeList>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct TransferEdgeData {
    edge:  &'static mut Edge,
    alpha: &'static EdgeAlpha,
    beta:  &'static EdgeBeta,
}

impl TransferEdgeDataItem<'_, '_> {
    fn split(&mut self) -> (&mut Edge, AlphaBeta<Entity>) {
        let alpha_beta = AlphaBeta { alpha: self.alpha.0, beta: self.beta.0 };
        (&mut self.edge, alpha_beta)
    }
}

fn transfer_system(
    conf: Res<Conf>,
    mut next_step: Local<u32>,
    mut storage_query: Query<TransferStorageData>,
    mut edge_query: Query<TransferEdgeData>,
    types: Res<Types>,
) {
    *next_step += 1;
    *next_step %= conf.transfer_timestep;
    if *next_step != 0 {
        return;
    }

    edge_query.par_iter_mut().for_each(|mut data| {
        let (edge, ab) = data.split();
        let Some(storages) = ab.map(|entity| storage_query.log_get(entity)).transpose() else {
            return;
        };

        compute_edge(&mut *edge, storages.map(|s| s.storage), &types, 1.0);
    });

    storage_query.par_iter_mut().for_each_init(
        || (0..types.types.len()).map(|_| ApplyStorageBufEntry::default()).collect::<Box<[_]>>(),
        |buf, mut storage| {
            apply_storage(
                buf,
                &mut storage.storage,
                storage.edges_alpha,
                storage.edges_beta,
                &edge_query,
                &types,
            );
        },
    );
}

fn compute_edge(edge: &mut Edge, storages: AlphaBeta<&Storage>, types: &Types, dt: f32) {
    // TODO insert field effects

    // advection is maximum number of moles transferred before considering fluidity
    let advective_base_rate = edge.resistance_recip * dt;
    let force_advection = edge.force_atob * advective_base_rate;
    let base_advection = if storages.map(|s| s.temperature).sum() > 0.0 {
        let pressure_gradient = storages.map(|s| s.pressure).net_diff();
        let unclamped_pressure_advection = pressure_gradient * edge.area * advective_base_rate;
        let advection_limit = pressure_gradient
            / PRESSURE_COEFFICIENT
            / storages.map(|s| s.temperature / s.volume).sum();
        unclamped_pressure_advection.clamp(-advection_limit.abs(), advection_limit.abs())
    } else {
        // if temperature is absolute zero, there is no pressure difference at all
        0.0
    } + force_advection;

    let contact_coef = edge.area * edge.resistance_recip * dt;
    let base_diffusion =
        contact_coef * storages.map(|s| s.temperature).sum() * 0.5 * DIFFUSION_COEFFICIENT;

    let mut conductivity = 0.0;
    let mut advective_convective_heat = Energy(0.0);

    for (type_id, (typed_pair, typed_edge)) in storages
        .map(|s| &s.types)
        .as_deref()
        .zip_iter()
        .zip(&mut edge.last_typed_transfer)
        .enumerate()
    {
        let type_id = TypeId(u32::try_from(type_id).expect("too many fluid types"));
        let type_def = types.get(type_id);

        // if advection is positive, alpha is the source
        let proportions = typed_pair.map(|t| t.proportion);
        let advection_source_moles = typed_pair.map(|t| t.moles).alpha_if(base_advection > 0.0);
        let src_proportion = proportions.alpha_if(base_advection > 0.0);
        let typed_advection = base_advection * src_proportion * type_def.advective_fluidity;

        let src_temp = storages.map(|s| s.temperature).alpha_if(base_advection > 0.0);
        let advective_heat = Energy(src_temp * typed_advection * type_def.molar_heat_capacity);

        let conc_pair = typed_pair.map(|t| t.molar_conc);
        let typed_diffusivity = base_diffusion * type_def.diffusive_fluidity;
        let typed_diffusion = conc_pair.map(|conc| Moles(conc * typed_diffusivity));
        let unclamped_diffusion = typed_diffusion.net_diff();
        let max_diffusion = conc_pair.net_diff() * storages.map(|s| s.volume).harmonic_mean();
        let net_diffusion = unclamped_diffusion.0.clamp(-max_diffusion.abs(), max_diffusion.abs());

        let convective_heat = Energy(
            type_def.molar_heat_capacity
                * typed_diffusion
                    .bimap(storages, |diffusion, s| diffusion.0 * s.temperature)
                    .net_diff(),
        );

        let prop_sum = proportions.sum();
        // this is supposed to be halved, but it is directly multiplied with type conductivity with
        // arbitrary unit anyway, so that's an unnecessary operation.
        conductivity += type_def.thermal_conductivity * prop_sum;
        advective_convective_heat += advective_heat + convective_heat;

        typed_edge.atob_transfer = Moles(
            (typed_advection + net_diffusion)
                .clamp(-advection_source_moles.0, advection_source_moles.0),
        );
    }

    let base_conduction = contact_coef * storages.map(|s| s.temperature).net_diff();
    let conductive_heat = base_conduction * conductivity;
    let max_conduction = storages.map(|s| s.temperature).net_diff()
        * storages.map(Storage::derived_total_heat_capacity).harmonic_mean();
    let clamped_conductive_heat =
        conductive_heat.clamp(-max_conduction.abs(), max_conduction.abs());
    edge.last_heat = Energy(clamped_conductive_heat) + advective_convective_heat;
}

#[derive(Debug, Clone, Default)]
struct ApplyStorageBufEntry {
    moles_change: Moles,
}

fn apply_storage(
    // buf for accumulating typed change over edges
    buf: &mut Box<[ApplyStorageBufEntry]>,
    storage: &mut Storage,
    edges_alpha: Option<&AlphaOfEdgeList>,
    edges_beta: Option<&BetaOfEdgeList>,
    edge_query: &Query<TransferEdgeData>,
    types: &Types,
) {
    buf.iter_mut().for_each(|entry| *entry = ApplyStorageBufEntry::default());

    let mut new_heat = storage.heat;

    for (edge_entity, sign) in iter::chain(
        edges_alpha.iter().flat_map(|list| &list.0).map(|&e| (e, -1.0)),
        edges_beta.into_iter().flat_map(|list| &list.0).map(|&e| (e, 1.0)),
    ) {
        let Some(edge) = edge_query.log_get(edge_entity) else { continue };
        let edge = edge.edge;

        new_heat += edge.last_heat * sign;

        for (typed_transfer, buf_entry) in edge.last_typed_transfer.iter().zip(&mut **buf) {
            let moles_change = typed_transfer.atob_transfer * sign;
            buf_entry.moles_change.0 += moles_change.0;
        }
    }
    storage.heat.0 = new_heat.0.max(0.0);
    // TODO add a cold path to deduct heat from peer connections

    for (typed_storage, buf_entry) in storage.types.iter_mut().zip(&**buf) {
        typed_storage.moles += buf_entry.moles_change;
        typed_storage.moles.0 = typed_storage.moles.0.max(0.0);
        // TODO add a cold path to deduct moles from peer connections when typed.moles < 0
    }

    let (total_moles, total_mass, total_heat_cap) =
        storage.types().fold((0.0, 0.0, 0.0), |(moles, mass, heat_cap), (type_id, typed)| {
            let type_def = types.get(type_id);
            (
                moles + typed.moles.0,
                mass + typed.moles.0 * type_def.molar_density,
                heat_cap + typed.moles.0 * type_def.molar_heat_capacity,
            )
        });

    let mut extinction = [0.0, 0.0, 0.0];
    // let mut emission = [0.0, 0.0, 0.0];
    let storage_volume = storage.volume;
    for (type_id, ty) in storage.types_mut() {
        ty.proportion =
            if ty.moles.0 == 0.0 || total_moles == 0.0 { 0.0 } else { ty.moles.0 / total_moles };
        ty.molar_conc = ty.moles.0 / storage_volume;

        #[expect(clippy::needless_range_loop, reason = "may be used for emission in the future")]
        for chan in 0..3 {
            let type_def = types.get(type_id);
            extinction[chan] += ty.proportion * type_def.optical_extinction[chan];
            // emission[chan] += ty.proportion * type_def.optical_emission[chan];
        }
    }

    storage.temperature = if total_heat_cap > 0.0 { storage.heat.0 / total_heat_cap } else { 0.0 };
    storage.mass = total_mass;
    storage.moles = Moles(total_moles);
    storage.pressure = total_moles * storage.temperature / storage.volume * PRESSURE_COEFFICIENT;

    storage.optical_extinction = extinction;
    // storage.optical_emission = emission;

    let mut transmittance_sum = 0.0;
    let rgb = [0, 1, 2].map(|chan| {
        let ambient = 1.0;
        let transmittance = (-extinction[chan] * storage.length).exp();
        transmittance_sum += transmittance;
        ambient * transmittance
    });
    let alpha = 1.0 - transmittance_sum / 3.0;

    storage.rgba = [rgb[0], rgb[1], rgb[2], alpha];
}

/// Component on viewers to track fluid type definition sync.
#[derive(Component, Reflect)]
pub struct ViewerSynced {
    num_types: usize,
}

fn sync_types_to_viewers_system(
    types: Res<Types>,
    viewers: Query<(Entity, Option<&ViewerSynced>), With<view::Viewer>>,
    mut commands: Commands,
    mut writer: MessageWriter<view::SentUpdate>,
) {
    for (entity, viewer) in viewers {
        if viewer.is_none_or(|v| v.num_types != types.types.len()) {
            commands.entity(entity).insert(ViewerSynced { num_types: types.types.len() });
            writer.write(view::SentUpdate {
                viewers: [entity].into(),
                body:    proto::Update::SetFluidTypes(proto::SetFluidTypes {
                    types: types
                        .types
                        .iter()
                        .map(|type_def| proto::FluidType { name: type_def.name.clone() })
                        .collect(),
                }),
            });
        }
    }
}
