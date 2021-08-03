//! Cargo definitions.

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Identifies a cargo type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TypeId(pub ArcStr);

/// A type of cargo.
#[derive(
    Debug, Clone, TypedBuilder, getset::CopyGetters, getset::Getters, Serialize, Deserialize,
)]
pub struct Type {
    /// Name of the cargo type.
    #[getset(get = "pub")]
    name: ArcStr,
    /// Short summary of the cargo type.
    #[getset(get = "pub")]
    summary: ArcStr,
    /// Long description of the cargo type.
    #[getset(get = "pub")]
    description: ArcStr,
    /// Category of the cargo type.
    #[getset(get = "pub")]
    category: CategoryId,
    /// The texture source path of the cargo.
    #[getset(get = "pub")]
    texture_src: ArcStr,
    /// The texture name of the cargo.
    #[getset(get = "pub")]
    texture_name: ArcStr,
}

/// Identifies a cargo category.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CategoryId(pub ArcStr);

/// A category of cargo.
#[derive(Debug, Clone, TypedBuilder, getset::Getters, Serialize, Deserialize)]
pub struct Category {
    /// Title of the cargo category.
    #[getset(get = "pub")]
    title: ArcStr,
    /// Description of the cargo category.
    #[getset(get = "pub")]
    description: ArcStr,
}
