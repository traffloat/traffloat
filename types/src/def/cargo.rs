//! Cargo definitions.

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

/// Identifies a cargo category
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeId(pub usize);

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
    #[getset(get_copy = "pub")]
    category: CategoryId,
    /// The texture source path of the cargo.
    #[getset(get = "pub")]
    texture_src: ArcStr,
    /// The texture name of the cargo.
    #[getset(get = "pub")]
    texture_name: ArcStr,
}

/// Identifies a cargo category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryId(pub usize);

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
