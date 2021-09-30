//! Skill definitions.

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Identifies a skill type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TypeId(pub ArcStr);

/// A type of skill.
#[derive(
    Debug, Clone, TypedBuilder, getset::CopyGetters, getset::Getters, Serialize, Deserialize,
)]
pub struct Type {
    /// Name of the skill type.
    #[getset(get = "pub")]
    name:        ArcStr,
    /// Long description of the skill type.
    #[getset(get = "pub")]
    description: ArcStr,
}
