use std::{cmp, iter, mem, ops};

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Local, Query, Res, SystemParam};

use crate::util::{AlphaBeta, MergeSortedItem, QueryExt, merge_sorted};

#[cfg(test)]
mod tests;

/// A constant equivalent to the ideal gas constant, used for pressure calculation.
pub const PRESSURE_COEFFICIENT: f32 = 1.0 / 128.0;

/// A constant to adjust the base diffusion rate.
pub const DIFFUSION_COEFFICIENT: f32 = 1.0 / 8192.0;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.init_resource::<Conf>();
        app.init_resource::<Types>();

        app.add_systems(app::FixedUpdate, transfer_system);
    }
}

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
)]
pub struct Energy(pub f32);

impl ops::Mul<f32> for Energy {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self { Self(self.0 * rhs) }
}

/// Identifies a fluid type, indexes [`Types::types`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(pub u32);

#[derive(Resource, Default)]
pub struct Types {
    pub types: Vec<TypeDef>,
}

impl Types {
    pub fn get(&self, ty: TypeId) -> &TypeDef {
        self.types.get(ty.0 as usize).expect("invalid fluid type reference created")
    }
}

pub struct TypeDef {
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
}

#[derive(Component, Debug, Clone)]
pub struct Storage {
    /// Volume provided by the storage.
    ///
    /// This may change over time subject to displacement,
    /// e.g. dumping cargo into a storage.
    ///
    /// May be mutated by modules defining a storage.
    pub volume: f32,

    /// Heat energy in the storage.
    pub heat:  Energy,
    /// Must always be sorted by type.
    // TODO benchmark possible alternative representations:
    // 1. fixed size vec with all types, even if zero
    // 2. use smallvec
    // 3. use dynamic components for each type
    pub types: Vec<TypedStorage>,

    // derived quantities
    /// Force per unit area exerted by the mixture.
    pub pressure:    f32,
    /// Absolute temperature.
    pub temperature: f32,
    /// Mass of fluid in this storage, used for force calculation in other modules.
    pub mass:        f32,
    /// Total moles of fluid in this storage.
    pub moles:       f32,
}

impl Storage {
    pub fn vacuum(volume: f32) -> Self {
        Self {
            volume,
            heat: Energy(0.0),
            types: Vec::new(),
            pressure: 0.0,
            temperature: 0.0,
            mass: 0.0,
            moles: 0.0,
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
        let index = self.types.partition_point(|t| t.ty < ty);
        if let Some(t) = self.types.get_mut(index)
            && t.ty == ty
        {
            t.moles = moles;
        } else {
            self.types.insert(index, TypedStorage { ty, moles, proportion: 0.0, molar_conc: 0.0 });
        }

        // for performance reasons,
        // we are not going to update proportion and molar conc until the next tick.
        // This is expected to have negligible impact
        // since it only affects flow rate multiplier computation for one tick.
    }

    pub fn total_heat_capacity(&self) -> f32 {
        if self.heat.0 == 0.0 || self.temperature == 0.0 {
            return 0.0;
        }
        self.heat.0 / self.temperature
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TypedStorage {
    pub ty:    TypeId,
    /// Amount of this type in this storage, in moles.
    pub moles: Moles,

    // derived quantities
    /// Proportion of volume in this storage occupied by this type, between 0 and 1.
    pub proportion: f32,
    /// Moles per unit volume of this type in this storage.
    pub molar_conc: f32,
}

/// The connection between two storages, through which fluid can transfer.
#[derive(Component, Debug, Clone)]
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
    pub fn new(resistance_recip: f32, area: f32) -> Self {
        Self {
            resistance_recip,
            area,
            force_atob: 0.0,
            last_heat: Energy(0.0),
            last_typed_transfer: Vec::new(),
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

#[derive(Debug, Clone)]
struct TypedTransfer {
    ty:            TypeId,
    atob_transfer: Moles,
}

#[derive(Component)]
#[relationship(relationship_target = AlphaOfEdgeList)]
pub struct EdgeAlpha(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = EdgeAlpha, linked_spawn)]
pub struct AlphaOfEdgeList(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = BetaOfEdgeList)]
pub struct EdgeBeta(pub Entity);

#[derive(Component)]
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

    storage_query.par_iter_mut().for_each_init(Default::default, |buf, mut storage| {
        apply_storage(
            buf,
            &mut storage.storage,
            storage.edges_alpha,
            storage.edges_beta,
            &edge_query,
            &types,
        );
    });
}

fn compute_edge(edge: &mut Edge, storages: AlphaBeta<&Storage>, types: &Types, dt: f32) {
    edge.last_typed_transfer.clear();
    edge.last_typed_transfer.reserve(storages.alpha.types.len().max(storages.beta.types.len()));

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

    for typed_pair in storages.map(|s| &s.types).merge_sorted(|t| t.ty) {
        let type_id = typed_pair.any(|typed| typed.ty);
        let type_def = types.get(type_id);

        // if advection is positive, alpha is the source
        let proportions = typed_pair.map(|t| t.proportion).default_ab();
        let advection_source_moles =
            typed_pair.map(|t| t.moles).default_ab().alpha_if(base_advection > 0.0);
        let src_proportion = proportions.alpha_if(base_advection > 0.0);
        let typed_advection = base_advection * src_proportion * type_def.advective_fluidity;

        let src_temp = storages.map(|s| s.temperature).alpha_if(base_advection > 0.0);
        let advective_heat = Energy(src_temp * typed_advection * type_def.molar_heat_capacity);

        let conc_pair = typed_pair.map(|t| t.molar_conc).default_ab();
        let typed_diffusibility = base_diffusion * type_def.diffusive_fluidity;
        let typed_diffusion = conc_pair.map(|conc| Moles(conc * typed_diffusibility));
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

        edge.last_typed_transfer.push(TypedTransfer {
            ty:            typed_pair.any(|t| t.ty),
            atob_transfer: Moles(
                (typed_advection + net_diffusion)
                    .clamp(-advection_source_moles.0, advection_source_moles.0),
            ),
        });
    }

    let base_conduction = contact_coef * storages.map(|s| s.temperature).net_diff();
    let conductive_heat = base_conduction * conductivity;
    let max_conduction = storages.map(|s| s.temperature).net_diff()
        * storages.map(|s| s.total_heat_capacity()).harmonic_mean();
    let clamped_conductive_heat =
        conductive_heat.clamp(-max_conduction.abs(), max_conduction.abs());
    edge.last_heat = Energy(clamped_conductive_heat) + advective_convective_heat;
}

struct ApplyStorageBufEntry {
    ty:           TypeId,
    moles_change: Moles,
}

fn apply_storage(
    (buf, swap): &mut (Vec<ApplyStorageBufEntry>, Vec<TypedStorage>),
    storage: &mut Storage,
    edges_alpha: Option<&AlphaOfEdgeList>,
    edges_beta: Option<&BetaOfEdgeList>,
    edge_query: &Query<TransferEdgeData>,
    types: &Types,
) {
    buf.clear();

    let mut new_heat = storage.heat;

    for (edge_entity, sign) in iter::chain(
        edges_alpha.iter().flat_map(|list| &list.0).map(|&e| (e, -1.0)),
        edges_beta.into_iter().flat_map(|list| &list.0).map(|&e| (e, 1.0)),
    ) {
        let Some(edge) = edge_query.log_get(edge_entity) else { continue };
        let edge = edge.edge;

        if let Some(additional) = edge.last_typed_transfer.len().checked_sub(buf.len()) {
            buf.reserve(additional);
        }

        new_heat += edge.last_heat * sign;

        let mut buf_index = 0;
        for typed_transfer in &edge.last_typed_transfer {
            let moles_change = typed_transfer.atob_transfer * sign;

            'skip_index: loop {
                let buf_entry = buf.get_mut(buf_index);
                buf_index += 1;

                if let Some(buf_entry) = buf_entry {
                    match buf_entry.ty.cmp(&typed_transfer.ty) {
                        cmp::Ordering::Less => continue 'skip_index,
                        cmp::Ordering::Equal => buf_entry.moles_change.0 += moles_change.0,
                        cmp::Ordering::Greater => buf.insert(
                            buf_index - 1,
                            ApplyStorageBufEntry { ty: typed_transfer.ty, moles_change },
                        ),
                    }
                } else {
                    buf.push(ApplyStorageBufEntry { ty: typed_transfer.ty, moles_change });
                }

                break;
            }
        }
    }
    storage.heat.0 = new_heat.0.max(0.0);
    // TODO add a cold path to deduct heat from peer connections

    mem::swap(swap, &mut storage.types);
    storage.types.clear();
    storage.types.extend(
        merge_sorted(&*swap, &*buf, |ty| ty.ty, |buf_entry| buf_entry.ty)
            .map(|item| {
                let ty = match item {
                    MergeSortedItem::Left(t) | crate::util::MergeSortedItem::Both(t, _) => t.ty,
                    MergeSortedItem::Right(buf_entry) => buf_entry.ty,
                };
                let moles = match item {
                    MergeSortedItem::Left(t) => t.moles,
                    MergeSortedItem::Right(e) => e.moles_change,
                    MergeSortedItem::Both(t, e) => t.moles + e.moles_change,
                };

                TypedStorage { ty, moles, molar_conc: 0.0, proportion: 0.0 }
            })
            .filter(|typed| typed.moles.0 > 0.0),
        // TODO add a cold path to deduct moles from peer connections when typed.moles < 0
    );

    let (total_moles, total_mass, total_heat_cap) =
        storage.types.iter().fold((0.0, 0.0, 0.0), |(moles, mass, heat_cap), typed| {
            let type_def = types.get(typed.ty);
            (
                moles + typed.moles.0,
                mass + typed.moles.0 * type_def.molar_density,
                heat_cap + typed.moles.0 * type_def.molar_heat_capacity,
            )
        });

    for ty in &mut storage.types {
        ty.proportion = ty.moles.0 / total_moles;
        ty.molar_conc = ty.moles.0 / storage.volume;
    }

    storage.temperature = if total_heat_cap > 0.0 { storage.heat.0 / total_heat_cap } else { 0.0 };
    storage.mass = total_mass;
    storage.moles = total_moles;
    storage.pressure = total_moles * storage.temperature / storage.volume * PRESSURE_COEFFICIENT;
}
