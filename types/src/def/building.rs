use typed_builder::TypedBuilder;

/// Identifies a building category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeId(pub usize);

/// A type of building.
#[derive(TypedBuilder, getset::CopyGetters, getset::Getters)]
pub struct Type {
    /// Name of the building type.
    #[getset(get = "pub")]
    name: String,
    /// Short summary of the building type.
    #[getset(get = "pub")]
    summary: String,
    /// Long description of the building type.
    #[getset(get = "pub")]
    description: String,
    /// Category of the building type.
    #[getset(get_copy = "pub")]
    category: CategoryId,
    /// Name of the texture.
    #[getset(get = "pub")]
    texture: String,
}

/// Identifies a building category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CategoryId(pub usize);

/// A category of building.
#[derive(TypedBuilder, getset::Getters)]
pub struct Category {
    /// Title of the building category.
    #[getset(get = "pub")]
    title: String,
    /// Description of the building category.
    #[getset(get = "pub")]
    description: String,
}
