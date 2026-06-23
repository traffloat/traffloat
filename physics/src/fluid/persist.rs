use serde::{Deserialize, Serialize};

use crate::fluid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEntry {
    pub heat:       f32,
    pub type_moles: Vec<f32>,
}

impl StorageEntry {
    pub fn from_component(comp: &fluid::Storage) -> Self {
        Self {
            heat:       comp.heat.0,
            type_moles: comp.types.iter().map(|typed| typed.moles.0).collect(),
        }
    }

    pub fn apply_to_component(&self, comp: &mut fluid::Storage) {
        comp.heat.0 = self.heat;
        for (src, dst) in self.type_moles.iter().zip(&mut comp.types) {
            dst.moles.0 = *src;
        }
    }
}
