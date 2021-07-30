//! Gas definitions.

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Identifies a gas category
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeId(pub usize);

/// A type of gas.
#[derive(
    Debug, Clone, TypedBuilder, getset::CopyGetters, getset::Getters, Serialize, Deserialize,
)]
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
    /// The texture source path of the gas.
    #[getset(get = "pub")]
    texture_src: ArcStr,
    /// The texture name of the gas.
    #[getset(get = "pub")]
    texture_name: ArcStr,
}
