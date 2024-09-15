//! Fluid definitions.

mod scalar;
mod types;

pub use scalar::{Save as SaveScalar, Scalar};
pub use types::{create_type, Save as SaveType, Type, TypeDef, Types};
