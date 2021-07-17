use typed_builder::TypedBuilder;

/// Identifies a cargo category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeId(pub usize);

/// A type of gas.
#[derive(Clone, TypedBuilder, getset::CopyGetters, getset::Getters)]
pub struct Type {
    /// Name of the gas type.
    #[getset(get = "pub")]
    name: String,
    /// Short summary of the gas type.
    #[getset(get = "pub")]
    summary: String,
    /// Long description of the gas type.
    #[getset(get = "pub")]
    description: String,
    /// Name of the texture.
    #[getset(get = "pub")]
    texture: String,
}
