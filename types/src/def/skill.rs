//! Skill definitions.

use arcstr::ArcStr;
use typed_builder::TypedBuilder;

/// Identifies a cargo category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeId(pub usize);

/// A type of skill.
#[derive(TypedBuilder, getset::CopyGetters, getset::Getters)]
pub struct Type {
    /// Name of the skill type.
    #[getset(get = "pub")]
    name: ArcStr,
    /// Long description of the skill type.
    #[getset(get = "pub")]
    description: ArcStr,
}
