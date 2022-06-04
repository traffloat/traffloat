//! A fluid is a liquid or gas material that can diffuse and mix with other fluids.

use serde::{Deserialize, Serialize};
use xylem::Xylem;

use crate::i18n::I18n;
use crate::unit;

/// Defines a fluid type.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub struct Fluid {
    /// The copy-safe identifier.
    #[xylem(args(new = true))]
    pub id:          Id,
    /// The string identifier.
    #[xylem(serde(default))]
    pub id_str:      IdString,
    /// The display name.
    pub name:        I18n,
    /// A short, one-line description.
    pub summary:     I18n,
    /// A detailed description.
    pub description: I18n,

    /// The viscosity of the fluid, affecting diffusion rate.
    pub viscosity:       unit::FluidViscosity,
    /// The compressibility of the fluid, affecting transport efficiency when containers are full.
    pub compressibility: unit::FluidCompressibility,
}

impl_identifiable!(Fluid);

/// Defines the fluid containers provided by a building.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub struct Storage {
    /// The copy-safe identifier.
    #[xylem(args(new = true))]
    pub id:      StorageId,
    /// The string identifier.
    #[xylem(serde(default))]
    pub id_str:  StorageIdString,
    /// The display name.
    pub name:    I18n,
    /// A short, one-line description.
    pub summary: I18n,
}

impl_identifiable!(@Storage Storage);
