use std::iter;

use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{Local, Query, Res, SystemParam};

use super::{
    AlphaOfEdgeList, BetaOfEdgeList, Conf, DIFFUSION_COEFFICIENT, Edge, EdgeAlpha, EdgeBeta,
    Energy, Moles, PRESSURE_COEFFICIENT, Storage, TypeId, Types,
};
use crate::util::{AlphaBeta, QueryExt};

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

#[derive(SystemParam)]
pub(super) struct TransferSystemParams<'w, 's> {
    conf:          Res<'w, Conf>,
    next_step:     Local<'s, u32>,
    storage_query: Query<'w, 's, TransferStorageData>,
    edge_query:    Query<'w, 's, TransferEdgeData>,
    types:         Res<'w, Types>,
}

pub(super) fn transfer_system(
    TransferSystemParams { conf, mut next_step, mut storage_query, mut edge_query, types }: TransferSystemParams,
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
