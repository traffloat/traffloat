//! Gas definitions.

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Identifies a gas category
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeId(pub usize);

/// A type of gas.
#[derive(Clone, TypedBuilder, getset::CopyGetters, getset::Getters, Serialize, Deserialize)]
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
