//! Gas definitions.

use codegen::Definition;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::atlas::Sprite;
use crate::lang;

/// A type of gas.
#[derive(Debug, Clone, CopyGetters, Getters, Serialize, Deserialize, Definition)]
pub struct Def {
    /// ID of the gas type.
    #[getset(get_copy = "pub")]
    id:          Id,
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
    texture:     Sprite,
}