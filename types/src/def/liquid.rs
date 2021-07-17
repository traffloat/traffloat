use typed_builder::TypedBuilder;

/// Identifies a cargo category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeId(pub usize);

/// A type of liquid.
#[derive(Clone, TypedBuilder, getset::Getters)]
pub struct Type {
    /// Name of the liquid type.
    #[getset(get = "pub")]
    name: String,
    /// Short summary of the liquid type.
    #[getset(get = "pub")]
    summary: String,
    /// Long description of the liquid type.
    #[getset(get = "pub")]
    description: String,
    /// Name of the texture.
    #[getset(get = "pub")]
    texture: String,
}
