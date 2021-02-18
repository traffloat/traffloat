use safety::Safety;
use traffloat::space::{Matrix, Vector};

pub type GlMatrix = nalgebra::Matrix4<f32>;
pub fn glize_matrix(mat: Matrix) -> GlMatrix {
    GlMatrix::from_iterator(mat.iter().map(|&f| f.lossy_trunc()))
}

pub type GlVector = nalgebra::Vector3<f32>;
pub fn glize_vector(vec: Vector) -> GlVector {
    GlVector::from_iterator(vec.iter().map(|&f| f.lossy_trunc()))
}
