use smallvec::SmallVec;
use specs::WorldExt;

use crate::types::*;
use crate::Setup;

/// Primitive unit shapes
#[derive(Debug, Clone, Copy, PartialEq, Eq, codegen::Gen)]
pub enum Unit {
    /// Thr unit sphere
    ///
    /// $$ B((0, 0, 0), 1) $$
    Sphere,
    /// The unit cylindee along the $z$-axis
    ///
    /// $$ B((0, 0), 1) \times (-1, q) $$
    Cylinder,
    /// A cube.
    ///
    /// $$ (-1, 1)^3 $$
    Cube,
    /// A regular tetrahedron
    ///
    /// A regular tetrahedron inscribed by the unit sphere,
    /// with one vertex on the positive $z$-axis.
    Tetra,
}

impl Unit {
    /// Checks whether the specified point is inside the unit
    pub fn contains(self, vector: &Vector) -> bool {
        match self {
            Unit::Sphere => vector.norm_squared() < 1.,
            Unit::Cylinder => vector.xy().norm_squared() < 1. && vector[2].abs() < 1.,
            Unit::Cube => vector[0].abs() < 1. && vector[1].abs() < 1. && vector[2].abs() < 1.,
            Unit::Tetra => todo!(),
        }
    }
}

/// A transformed primitive shape
#[derive(Debug, Clone, PartialEq, Component, codegen::Gen)]
#[storage(storage::DenseVecStorage)]
pub struct Shape {
    /// The base unit shape
    pub unit: Unit,
    /// The transformation matrix
    pub transform: Matrix,
}

impl Shape {
    /// Checks whether the shape is in the clipping space
    pub fn is_clipped(&self) -> bool {
        true // TODO implement
    }
}

pub fn setup_specs((mut world, dispatcher): Setup) -> Setup {
    world.register::<Shape>();
    (world, dispatcher)
}
