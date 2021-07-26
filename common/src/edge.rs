//! Edge management.
//!
//! An edge is also called a "corridor".
//! It connects two nodes together.

use derive_new::new;
use legion::Entity;
use typed_builder::TypedBuilder;

use crate::space::{Matrix, Position, Vector};
use crate::SetupEcs;

/// Component storing the endpoints of an edge
#[derive(Debug, Clone, Copy, PartialEq, Eq, new, getset::CopyGetters, getset::Setters)]
pub struct Id {
    /// The "source" node
    #[getset(get_copy = "pub")]
    from: Entity,
    /// The "dest" node
    #[getset(get_copy = "pub")]
    to: Entity,
}

/// Defines the size of an edge
#[derive(Debug, Clone, Copy, new, getset::CopyGetters)]
pub struct Size {
    /// The radius of the corridor
    #[getset(get_copy = "pub")]
    radius: f64,
}

/// A position on the cross section of an edge.
#[derive(Debug, Clone, Copy, Default, new)]
pub struct CrossSectionPosition(nalgebra::Vector2<f64>);

impl CrossSectionPosition {
    /// The vector from the center to the position.
    pub fn vector(self) -> nalgebra::Vector2<f64> {
        self.0
    }
}

/// The geometric design of the edge.
///
/// This is only used for graphical user interaction.
/// Simulation systems should depend on separate components on capacity
/// instead of calculating from this data structure.
#[derive(Debug, new, getset::Getters)]
pub struct Design {
    /// The ducts in the edge.
    #[getset(get = "pub")]
    ducts: Vec<Duct>,
}

/// A circular structure in an edge.
///
/// The actual content of the duct is stored in the referred entity.
#[derive(Debug, TypedBuilder, getset::Getters, getset::CopyGetters)]
pub struct Duct {
    /// The center of a circle.
    #[getset(get_copy = "pub")]
    center: CrossSectionPosition,
    /// The radius of a circle.
    #[getset(get_copy = "pub")]
    radius: f64,
    /// The entity storing the duct attributes.
    #[getset(get_copy = "pub")]
    entity: Entity,
}

/// Indicates that an edge is added
#[derive(Debug, new, getset::Getters)]
pub struct AddEvent {
    /// The added edge
    #[getset(get = "pub")]
    edge: Id,
}

/// Indicates that an edge is flagged for removal
#[derive(Debug, new, getset::Getters)]
pub struct RemoveEvent {
    /// The removed edge
    #[getset(get = "pub")]
    edge: Id,
}
/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup
}

/// Computes the transformation matrix from or to the unit cylinder
pub fn tf(edge: &Id, size: &Size, world: &legion::world::SubWorld, from_unit: bool) -> Matrix {
    use legion::EntityStore;

    let from = edge.from();
    let to = edge.to();

    let from: Position = *world
        .entry_ref(from)
        .expect("from_entity does not exist")
        .get_component()
        .expect("from node does not have Position");
    let to: Position = *world
        .entry_ref(to)
        .expect("to_entity does not exist")
        .get_component()
        .expect("to node does not have Position");

    let dir = to - from;
    let rot = match nalgebra::Rotation3::rotation_between(&Vector::new(0., 0., 1.), &dir) {
        Some(rot) => rot.to_homogeneous(),
        None => Matrix::identity().append_nonuniform_scaling(&Vector::new(0., 0., -1.)),
    };

    if from_unit {
        rot.prepend_nonuniform_scaling(&Vector::new(size.radius(), size.radius(), dir.norm()))
            .append_translation(&from.vector())
    } else {
        rot.transpose()
            .prepend_translation(&-from.vector())
            .append_nonuniform_scaling(&Vector::new(
                1. / size.radius(),
                1. / size.radius(),
                1. / dir.norm(),
            ))
    }
}

/// Return type of [`create_components`].
pub type Components = (Id, Size, Design);

/// Creates the components for a node entity.
pub fn create_components(from: Entity, to: Entity, size: f64) -> Components {
    (Id::new(from, to), Size::new(size), Design::new(Vec::new()))
}
