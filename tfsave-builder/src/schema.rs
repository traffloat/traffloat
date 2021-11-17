use std::path::PathBuf;

use serde::Deserialize;
use traffloat_def::{AnyDefXylem, Config, Scenario};

use crate::init::{InitXylem, ScalarState};

/// The schema in the main.toml file.
#[derive(Deserialize)]
pub struct MainFile {
    /// The extra schema in the main.toml file.
    #[serde(flatten)]
    pub main: Main,
    /// The typical schema common in all files.
    #[serde(flatten)]
    pub file: File,
}

/// The extra schema in the main.toml file.
#[derive(Deserialize)]
pub struct Main {
    /// Scenario metadata.
    pub scenario: Scenario,
    /// Scalar configuration for this scenario.
    pub config:   Config,
    /// Scalar game states.
    pub state:    ScalarState,
}

/// The schema for included TOML files.
#[derive(Deserialize)]
pub struct File {
    /// Extra files to include.
    ///
    /// All includes are resolved before definitions.
    /// In other words, include is performed in depth-first order.
    #[serde(default)]
    pub include: Vec<Include>,
    /// Gamerules defined in this file.
    #[serde(default)]
    pub def:     Vec<AnyDefXylem>,
    /// Initialize states.
    #[serde(default)]
    pub init:    Vec<InitXylem>,
}

/// References another file to include.
#[derive(Deserialize)]
pub struct Include {
    /// The path to include.
    pub file: PathBuf,
}
