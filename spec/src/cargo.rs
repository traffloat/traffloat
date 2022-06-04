//! A cargo is a portable item type that can be carried by vehicles or inhabitants directly.

use serde::{Deserialize, Serialize};
use xylem::Xylem;

use crate::i18n::I18n;

/// Defines a fluid type.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub struct Cargo {
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

    /// The mass per unit volume.
    pub density: f32,
}

impl_identifiable!(Cargo);
