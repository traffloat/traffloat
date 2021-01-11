use super::types::*;

/// A transformed primitive shape
#[derive(codegen::Gen)]
pub struct Shape {
    /// The base unit shape
    pub unit: Unit,
    /// The transformation matrix
    pub transform: Matrix,
}

/// Primitive unit shapes
#[derive(codegen::Gen)]
pub enum Unit {
    /// Thr unit sphere
    ///
    /// $$ B((0, 0, 0), 1) $$
    Sphere,
    /// The unit cylindee along the $z$-axis
    ///
    /// $$ B((0, 0), 1) \times [-1, q] $$
    Cylinder,
    /// A cube.
    ///
    /// $$ [-1, 1]^3 $$
    Cube,
    /// A regular tetrahedron
    ///
    /// A regular tetrahedron inscribed by the unit sphere,
    /// with one vertex on the positive $z$-axis.
    Tetra,
}
