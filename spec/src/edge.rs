//! An edge is a corridor that connects two nodes.

use serde::{Deserialize, Serialize};
use xylem::Xylem;

use crate::i18n::I18n;

/// Creates a edge.
#[derive(Debug, Clone, Serialize, Deserialize, Xylem)]
#[xylem(derive(Deserialize))]
pub struct Edge {}
