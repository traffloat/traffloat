//! Liquid definitions.

use arcstr::ArcStr;
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
    name: ArcStr,
    /// Short summary of the liquid type.
    #[getset(get = "pub")]
    summary: ArcStr,
    /// Long description of the liquid type.
    #[getset(get = "pub")]
    description: ArcStr,
    /// Viscosity of a liquid.
    #[getset(get = "pub")]
    viscosity: units::LiquidViscosity,
    /// Name of the texture.
    #[getset(get = "pub")]
    texture: ArcStr,
}
