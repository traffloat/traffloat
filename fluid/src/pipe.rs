//! Implements fluid ducts ("pipes").
//!
//! # Systems
//! 1. Compute fluid-independent flow rate factor.
//! 2. Compute fluid-dependent flow rate and update storages.
//! 3. Update the total volume and pressure for each storage.

use dynec::{archetype, comp, system, Entity};
use rayon::iter::ParallelIterator;
use traffloat_base::DeltaTime;
use traffloat_graph::edge::{EndpointValues, WhichEndpoint};
use traffloat_graph::{edge, Duct, Node};

use crate::storage::{self, Storage};
use crate::{Mass, Type, TypeDef, TypeDefs, Volume};

const FLOW_COEFFICIENT: f64 = 1.0;

archetype! {
    /// A pipe transfers fluids from a storage in a node to a storage in an adjacent node.
    ///
    /// A pipe either uniquely occupies a duct or delivers between two storages of the same node.
    pub Pipe;
}

/// Extension component to reference the pipe details from the generic duct.
#[comp(of = Duct)]
pub struct RefFromDuct(#[entity] pub Entity<Pipe>);

/// Extension component to list the intra-node pipes in a node.
#[comp(of = Node)]
pub struct IntraPipeList(#[entity] pub Vec<Entity<Pipe>>);

/// Back-reference to the owning duct, or the node for intra-node ducts.
#[comp(of = Pipe, required)]
pub enum OwnerRef {
    /// The duct corresponding to the pipe.
    Duct(#[entity] Entity<Duct>),
    /// The node corresponding to the intra-node storage-connecting pipe.
    Node(#[entity] Entity<Node>),
}

/// Radius of the pipe.
///
/// This value may be different from the duct radius since pipe wall is not included here.
#[comp(of = Pipe, required)]
pub struct Radius(f64);

impl Radius {
    pub fn get(&self) -> f64 { self.0 }

    pub fn set(&mut self, value: f64, length: &edge::Length, resist: &mut FlowResistance) {
        self.0 = value;
        resist.quantity = length.quantity / value.powi(4);
    }
}

/// Pipe length divided by 4th power of pipe radius, cached for simulation performance.
///
/// Updates to [`Radius`] should go through [`Radius::set`] to update cached computations together.
#[dynec::comp(of = Pipe, required)]
pub struct FlowResistance {
    pub quantity: f64,
}

/// The cross section area of the pipe, used in flow rate calculation.
#[comp(of = Pipe, required)]
pub struct CrossSectionArea {
    pub quantity: f64,
}

/// The storages that the duct actually connects.
#[comp(of = Pipe, required)]
pub struct Endpoints {
    /// The storage in alpha, or the "alpha storage" if both storages are in the same node.
    #[entity]
    pub alpha: Entity<Storage>,
    /// The storage in beta, or the "beta storage" if both storages are in the same node.
    #[entity]
    pub beta:  Entity<Storage>,
}

impl Endpoints {
    pub fn which(&self, which: WhichEndpoint) -> &Entity<Storage> {
        match which {
            WhichEndpoint::Alpha => &self.alpha,
            WhichEndpoint::Beta => &self.beta,
        }
    }
}

/// The viscosity-independent factor affecting the flow rate from alpha to beta.
/// The actual flow rate should be proportional to factor \* relative_volume / viscosity.
#[comp(of = Pipe, required, init = || FlowFactor{factor: 0.})]
struct FlowFactor {
    pub factor: f64,
}

/// Partition after [`compute_flow_factor`] is executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BeforeComputeFlowFactor;

/// Partition after [`compute_flow_factor`] is executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AfterComputeFlowFactor;

#[system(after(BeforeComputeFlowFactor), before(AfterComputeFlowFactor))]
pub fn compute_flow_factor(
    #[dynec(global)] dt: &DeltaTime,
    duct_iter: system::EntityIterator<Pipe>,
    endpoints_read: system::ReadSimple<Pipe, Endpoints>,
    resistance_read: system::ReadSimple<Pipe, FlowResistance>,
    pressure_read: system::ReadSimple<Storage, storage::CurrentPressure>,
    mut flow_factor_write: system::WriteSimple<Pipe, FlowFactor>,
) {
    duct_iter
        .par_entities_with_chunked((&endpoints_read, &resistance_read, &mut flow_factor_write))
        .for_each(|(_, (endpoints, resistance, flow_factor))| {
            let alpha_pressure = pressure_read.get(&endpoints.alpha).pressure;
            let beta_pressure = pressure_read.get(&endpoints.beta).pressure;
            let pressure_diff = alpha_pressure - beta_pressure; // +ve means alpha pushing towards beta

            // Hagen–Poiseuille law: flow rate = constant * r^4 * pressure_diff / length / viscosity
            flow_factor.factor =
                pressure_diff.quantity / resistance.quantity * dt.quantity * FLOW_COEFFICIENT;
        })
}

/// Partition after [`transfer_duct`] is executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AfterTransferDuct;

/// The mass of a type of fluid flowing through a pipe in the current tick.
#[comp(of = Pipe, isotope = Type)]
pub struct DeltaMass(Mass);

/// A dynec system that transfers fluids across ducts.
#[system(after(AfterComputeFlowFactor), before(AfterTransferDuct))]
pub fn transfer_duct(
    #[dynec(param)] ty: &Type,
    #[dynec(global)] defs: &TypeDefs,
    duct_iter: system::EntityIterator<Pipe>,
    endpoints_read: system::ReadSimple<Pipe, Endpoints>,
    flow_factor_read: system::ReadSimple<Pipe, FlowFactor>,
    #[dynec(isotope(discrim = [ty]))] mut mass_write: system::WriteIsotopePartial<
        Storage,
        storage::TypedMass,
        [Type; 1],
    >,
    #[dynec(isotope(discrim = [ty]))] mut volume_read: system::ReadIsotopePartial<
        Storage,
        storage::TypedVolume,
        [Type; 1],
    >,
) {
    let &TypeDef { viscosity, .. } = defs.get(*ty);
    let [mut mass_write] = mass_write.split_mut([0]);
    let [volume_read] = volume_read.split([0]);

    // we can parallelize this by summing up the delta within each worker
    // and add them up in a separate parallelized loop over all storages,
    // but this appears to be unnecessary since the number of `transfer_duct` systems
    // may be comparable to the amount of parallelism available.
    duct_iter.entities_with_chunked((&endpoints_read, &flow_factor_read)).for_each(
        |(_, (endpoints, flow_factor))| {
            let source = WhichEndpoint::alpha_if(flow_factor.factor > 0.);

            let mass = EndpointValues::from_array(
                mass_write.get_many_mut([&endpoints.alpha, &endpoints.beta]),
            );
            let volume = EndpointValues::from_array([
                volume_read.get(&endpoints.alpha),
                volume_read.get(&endpoints.beta),
            ]);
            let conc =
                mass.bimap(&volume, |mass, volume| mass.mass.quantity / volume.volume.quantity);
            let conc_gradient = conc.alpha - conc.beta;

            let flow_volume = flow_factor.factor * viscosity.quantity;

            let delta_mass = Mass {
                quantity: (flow_volume * conc.get_ref(source)).clamp(
                    -mass.get_ref(WhichEndpoint::Beta).mass.quantity,
                    mass.get_ref(WhichEndpoint::Alpha).mass.quantity,
                ),
            };
            mass.alpha.mass -= delta_mass;
            mass.beta.mass += delta_mass;
        },
    );
}

/// Partition after [`rebalance_pv`] is executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AfterRebalanceStorages;

/// Updates the pressure and density of each storage after performing transfer.
#[system(after(AfterTransferDuct), before(AfterRebalanceStorages))]
pub fn rebalance_storages(
    storage_iter: system::EntityIterator<Storage>,
    #[dynec(global)] defs: &TypeDefs,
    mut volume_sum_write: system::WriteSimple<Storage, storage::VolumeSum>,
    mut pressure_write: system::WriteSimple<Storage, storage::CurrentPressure>,
    mut current_mass_read: system::ReadIsotopeFull<Storage, storage::TypedMass>,
    mut current_volume_read: system::ReadIsotopeFull<Storage, storage::TypedVolume>,
) {
    storage_iter.par_entities_with_chunked(&mut volume_sum_write).for_each(|(_, total)| {
        total.volume = Volume { quantity: 0.0 };
    });

    for (ty, _) in defs.iter() {
        let [ty_volume_read] = current_volume_read.split([ty]);
        storage_iter.par_entities_with_chunked((&mut volume_sum_write, &ty_volume_read)).for_each(
            |(_, (total, current))| {
                total.volume += current.volume;
            },
        );
    }

    storage_iter
        .par_entities_with_chunked((&volume_sum_write, &mut pressure_write))
        .for_each(|(_, (volume, pressure))| {});
}
