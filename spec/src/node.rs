//! A node is an instance of a building.

use serde::{Deserialize, Serialize};
use xylem::Xylem;

use crate::i18n::I18n;

/// Creates a node.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub struct Node {
    /// The copy-safe identifier.
    #[xylem(args(new = true))]
    pub id:     Id,
    /// The string identifier.
    #[xylem(serde(default))]
    pub id_str: IdString,
}

impl_identifiable!(Node);
