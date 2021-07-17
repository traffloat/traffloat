use typed_builder::TypedBuilder;

/// Identifies a cargo category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeId(pub usize);

/// A type of cargo.
#[derive(TypedBuilder, getset::CopyGetters, getset::Getters)]
pub struct Type {
    /// Name of the cargo type.
    #[getset(get = "pub")]
    name: String,
    /// Short summary of the cargo type.
    #[getset(get = "pub")]
    summary: String,
    /// Long description of the cargo type.
    #[getset(get = "pub")]
    description: String,
    /// Category of the cargo type.
    #[getset(get_copy = "pub")]
    category: CategoryId,
    /// Name of the texture.
    #[getset(get = "pub")]
    texture: String,
}

/// Identifies a cargo category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CategoryId(pub usize);

/// A category of cargo.
#[derive(TypedBuilder, getset::Getters)]
pub struct Category {
    /// Title of the cargo category.
    #[getset(get = "pub")]
    title: String,
    /// Description of the cargo category.
    #[getset(get = "pub")]
    description: String,
}
