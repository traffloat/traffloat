//! Fluid definitions.

mod scalar;
mod types;

pub use scalar::{Save as SaveScalar, Scalar};
pub use types::{create_type, CreatedType, OnCreateType, Save as SaveType, Type, TypeDef, Types};
