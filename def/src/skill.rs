//! Skill definitions.

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::{lang, IdString};

/// Identifies a skill type.
pub type Id = crate::Id<Def>;

impl_identifiable!(Def);

/// A type of skill.
#[derive(Debug, Clone, CopyGetters, Getters, Serialize, Deserialize)]
#[cfg_attr(feature = "xy", derive(xylem::Xylem))]
#[cfg_attr(feature = "xy", xylem(derive(Deserialize)))]
pub struct Def {
    /// ID of the skill type.
    #[getset(get_copy = "pub")]
    #[cfg_attr(feature = "xy", xylem(args(new = true)))]
    id:          Id,
    /// String ID of the skill type.
    #[getset(get = "pub")]
    #[cfg_attr(feature = "xy", xylem(serde(default)))]
    id_str:      IdString<Def>,
    /// Name of the skill type.
    #[getset(get = "pub")]
    name:        lang::Item,
    /// Long description of the skill type.
    #[getset(get = "pub")]
    description: lang::Item,
}
