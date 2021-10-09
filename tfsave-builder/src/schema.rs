use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use traffloat_def::Definition;

/// The schema for the main.toml file.
#[derive(Serialize, Deserialize)]
pub struct Main {
    /// Scenario metadata.
    pub scenario: Scenario,
    /// Scalar configuration for this scenario.
    pub config:   Config,
    /// The includes in the main file.
    pub include:  Vec<Include>,
}

/// The schema for included TOML files.
#[derive(Serialize, Deserialize)]
pub struct File {
    /// Extra files to include.
    ///
    /// All includes are resolved before definitions.
    /// In other words, include is performed in depth-first order.
    pub include: Vec<Include>,
    /// Gamerules defined in this file.
    pub def:     Vec<Definition>,
}

/// References another file to include.
#[derive(Serialize, Deserialize)]
pub struct Include {
    /// The path to include.
    pub file: PathBuf,
}
