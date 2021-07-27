//! Liquid definitions.

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::units;

/// Identifies a liquid category
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeId(pub usize);

/// A type of liquid.
#[derive(Debug, Clone, TypedBuilder, getset::Getters, Serialize, Deserialize)]
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
