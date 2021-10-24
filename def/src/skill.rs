//! Skill definitions.

use codegen::{Definition, IdStr};
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::lang;

/// A type of skill.
#[derive(Debug, Clone, CopyGetters, Getters, Serialize, Deserialize, Definition)]
pub struct Def {
    /// ID of the skill type.
    #[getset(get_copy = "pub")]
    id:          Id,
    /// String ID of the skill type.
    #[getset(get = "pub")]
    id_str:      IdStr,
    /// Name of the skill type.
    #[getset(get = "pub")]
    name:        lang::Item,
    /// Long description of the skill type.
    #[getset(get = "pub")]
    description: lang::Item,
}
