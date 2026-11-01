use serde::{Deserialize, Serialize};

use crate::cargo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEntry {
    pub type_granules: Vec<i64>,
}

impl StorageEntry {
    pub fn from_component(comp: &cargo::Storage) -> Self {
        Self { type_granules: comp.types().map(|typed| typed.granules.0).collect() }
    }

    pub fn apply_to_component(&self, comp: &mut cargo::Storage) {
        for (&src, dst) in self.type_granules.iter().zip(comp.types_mut()) {
            dst.granules.0 = src;
        }
    }
}
