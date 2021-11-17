//! Defines features of a node.

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use traffloat_types::units::{self, Unit};

use crate::catalyst::Catalyst;
use crate::Schema;

pub mod reaction;
pub use reaction::Reaction;
pub mod housing;
pub mod security;
pub use housing::Housing;

/// Features of a building.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize), serde(tag = "type")))]
pub enum Feature {
    /// The building is a core and must not be destroyed.
    Core,
    /// The building provides housing capacity, and inhabitants can be assigned to it.
    ProvidesHousing(Housing),
    /// The building performs a reaction.
    Reaction(Reaction),
    /// The building provides driving force for vehicles on adjacent rails.
    RailPump(PumpSpec<units::RailForce>),
    /// The building provides pumping force for adjacent liquid pipes.
    LiquidPump(PumpSpec<units::PipeForce>),
    /// The building provides pumping force for gas diffusion in adjacent corridors.
    GasPump(PumpSpec<units::FanForce>),
    /// Inhabitants with low skill may not be permitted to enter the node.
    SecureEntry(security::Policy),
    /// Inhabitants with low skill may not be permitted to exit the node.
    SecureExit(security::Policy),
}

/// Describes a generic pump.
#[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct PumpSpec<U: Unit + 'static> {
    /// Catalysts affecting the pump efficiency.
    #[getset(get = "pub")]
    catalysts: SmallVec<[Catalyst; 2]>,
    /// The base force provided by the pump.
    #[getset(get_copy = "pub")]
    force:     U,
}

#[cfg(feature = "xy")]
const _: () = {
    use xylem::{DefaultContext, NoArgs, Xylem};

    impl<U: Unit + Xylem<Schema> + 'static> Xylem<Schema> for PumpSpec<U> {
        type From = PumpSpecXylem<U>;
        type Args = NoArgs;

        fn convert_impl(
            from: Self::From,
            context: &mut DefaultContext,
            _: &NoArgs,
        ) -> anyhow::Result<Self> {
            Ok(Self {
                catalysts: SmallVec::convert(from.catalysts, context, &NoArgs)?,
                force:     from.force,
            })
        }
    }

    /// See [`PumpSpec`].
    #[derive(Deserialize)]
    #[serde(bound = "")]
    pub struct PumpSpecXylem<U: Unit> {
        #[serde(default)]
        catalysts: Vec<<Catalyst as Xylem<Schema>>::From>,
        force:     U,
    }
};
