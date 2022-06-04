//! An attribute is a scalar quantity describing a node.

use serde::{Deserialize, Serialize};
use xylem::Xylem;

use crate::i18n::I18n;

/// Defines an attribute.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub struct Attribute {
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
}

impl_identifiable!(Attribute);
