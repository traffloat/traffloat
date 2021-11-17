use serde::{Deserialize, Serialize};
use traffloat_types::space::{Matrix, Vector};
use xylem::{DefaultContext, NoArgs, Xylem};

use crate::Schema;

impl Xylem<Schema> for Matrix {
    type From = Vec<TransformPrimitive>;
    type Args = NoArgs;

    fn convert_impl(
        from: Self::From,
        _: &mut DefaultContext,
        _: &Self::Args,
    ) -> anyhow::Result<Self> {
        let mut matrix = Matrix::identity();
        for primitive in from {
            matrix = primitive.to_matrix() * matrix;
        }
        Ok(matrix)
    }
}

/// A primitive linear transformation operation.
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransformPrimitive {
    /// A translation operation.
    Translate {
        /// The distance to translate along the X axis. Default 0.
        #[serde(default)]
        x: f64,
        /// The distance to translate along the Y axis. Default 0.
        #[serde(default)]
        y: f64,
        /// The distance to translate along the Z axis. Default 0.
        #[serde(default)]
        z: f64,
    },
    /// A scaling operation.
    Scale {
        /// The ratio to scale along the X axis. Default 1.
        #[serde(default = "serde_one")]
        x: f64,
        /// The ratio to scale along the Y axis. Default 1.
        #[serde(default = "serde_one")]
        y: f64,
        /// The ratio to scale along the Z axis. Default 1.
        #[serde(default = "serde_one")]
        z: f64,
    },
}

impl TransformPrimitive {
    /// Represent this primitive as a transformation matrix.
    fn to_matrix(&self) -> Matrix {
        match *self {
            Self::Translate { x, y, z } => Matrix::new_translation(&Vector::new(x, y, z)),
            Self::Scale { x, y, z } => Matrix::new_nonuniform_scaling(&Vector::new(x, y, z)),
        }
    }
}

fn serde_one() -> f64 { 1. }
