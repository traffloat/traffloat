//! Language bundle definitions

use std::collections::BTreeMap;

use arcstr::ArcStr;
use codegen::Definition;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

/// A bundle of language files.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, CopyGetters, Definition)]
pub struct Def {
    /// Identifies the language bundle.
    #[getset(get_copy = "pub")]
    id:        Id,
    /// Paths to language files.
    #[getset(get = "pub")]
    languages: BTreeMap<ArcStr, ArcStr>,
}

/// A translatable message template.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, CopyGetters, Definition)]
pub struct Item {
    /// The language bundle to use.
    #[getset(get_copy = "pub")]
    src: Id,
    /// The key for the string in the language bundle.
    #[getset(get = "pub")]
    key: ArcStr,
}
