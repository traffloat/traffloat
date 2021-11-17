//! Gas definitions.

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::atlas::IconRef;
use crate::{lang, IdString};

/// Identifies a gas type.
pub type Id = crate::Id<Def>;

impl_identifiable!(Def);

/// A type of gas.
#[derive(Debug, Clone, CopyGetters, Getters, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct Def {
    /// ID of the gas type.
    #[getset(get_copy = "pub")]
    #[cfg_attr(feature = "xy", xylem(args(new = true)))]
    id:          Id,
    /// String ID of the gas type.
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    id_str:      IdString<Def>,
    /// Name of the gas type.
    #[getset(get = "pub")]
    name:        lang::Item,
    /// Short summary of the gas type.
    #[getset(get = "pub")]
    summary:     lang::Item,
    /// Long description of the gas type.
    #[getset(get = "pub")]
    description: lang::Item,
    /// The texture of the gas.
    #[getset(get = "pub")]
    texture:     IconRef,
}
