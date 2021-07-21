//! Liquid definitions.

use typed_builder::TypedBuilder;

use crate::units;

/// Identifies a liquid category
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
    /// Viscosity of a liquid.
    #[getset(get = "pub")]
    viscosity: units::LiquidViscosity,
    /// Name of the texture.
    #[getset(get = "pub")]
    texture: String,
}
