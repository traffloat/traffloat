//! Gas definitions.

use arcstr::ArcStr;
use typed_builder::TypedBuilder;

/// Identifies a gas category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeId(pub usize);

/// A type of gas.
#[derive(Clone, TypedBuilder, getset::CopyGetters, getset::Getters)]
pub struct Type {
    /// Name of the gas type.
    #[getset(get = "pub")]
    name: ArcStr,
    /// Short summary of the gas type.
    #[getset(get = "pub")]
    summary: ArcStr,
    /// Long description of the gas type.
    #[getset(get = "pub")]
    description: ArcStr,
    /// Name of the texture.
    #[getset(get = "pub")]
    texture: ArcStr,
}
