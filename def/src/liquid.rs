//! Liquid definitions.

use codegen::Definition;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use traffloat_types::units;

use crate::atlas::IconRef;
use crate::lang;

/// A type of liquid.
#[derive(Debug, Clone, Getters, CopyGetters, Serialize, Deserialize, Definition)]
pub struct Def {
    /// ID of the liquid type.
    #[getset(get_copy = "pub")]
    id:          Id,
    /// Name of the liquid type.
    #[getset(get = "pub")]
    name:        lang::Item,
    /// Short summary of the liquid type.
    #[getset(get = "pub")]
    summary:     lang::Item,
    /// Long description of the liquid type.
    #[getset(get = "pub")]
    description: lang::Item,
    /// Viscosity of a liquid.
    #[getset(get_copy = "pub")]
    viscosity:   units::LiquidViscosity,
    /// The texture of the liquid.
    #[getset(get = "pub")]
    texture:     IconRef,
}

/// A formula for mixing liquids.
///
/// Formulas are always commutative, i.e. `a + b = b + a`;
/// the commutation is automatically filled.
///
/// To ensure reproducibility, formulas should be associative,
/// i.e. `a + (b + c) = (a + b) + c`.
#[derive(Debug, Clone, CopyGetters, Serialize, Deserialize, Definition)]
pub struct Formula {
    /// One of the ingredients for mixing.
    #[getset(get_copy = "pub")]
    augend: Id,
    /// One of the ingredients for mixing.
    #[getset(get_copy = "pub")]
    addend: Id,
    /// The output after mixing.
    #[getset(get_copy = "pub")]
    sum:    Id,
}

/// The default output if no corresponding formula is defined.
#[derive(Debug, Clone, CopyGetters, Serialize, Deserialize, Definition)]
pub struct DefaultFormula {
    /// The output after mixing.
    #[getset(get_copy = "pub")]
    sum: Id,
}
