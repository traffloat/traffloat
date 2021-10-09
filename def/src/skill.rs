//! Skill definitions.

use codegen::Definition;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::lang;

/// A type of skill.
#[derive(Debug, Clone, CopyGetters, Getters, Serialize, Deserialize, Definition)]
pub struct Def {
    /// ID of the skill type.
    #[getset(get_copy = "pub")]
    id:          Id,
    /// Name of the skill type.
    #[getset(get = "pub")]
    name:        lang::Item,
    /// Long description of the skill type.
    #[getset(get = "pub")]
    description: lang::Item,
}
