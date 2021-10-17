//! Housing definitions.

use codegen::Definition;
use getset::CopyGetters;
use serde::{Deserialize, Serialize};

use crate::building::storage;

/// Defines a housing feature.
#[derive(Debug, Clone, CopyGetters, Serialize, Deserialize, Definition)]
pub struct Housing {
    /// The population storage that the housing is located in.
    #[getset(get_copy = "pub")]
    storage: storage::population::Id,
}
