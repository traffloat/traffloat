//! Edge management.
//!
//! An edge is also called a "corridor".
//! It connects two nodes together.

use derive_new::new;
use legion::{systems::CommandBuffer, world::SubWorld, Entity};
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::node;
use crate::space::{Matrix, Position, Vector};
use crate::units;
use crate::SetupEcs;
use crate::{cargo, gas, liquid};

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

codegen::component_depends! {
    Id = (
        Id,
        Size,
        units::Portion<units::Hitpoint>,
        Design,
    ) + ?(
    )
}

/// Defines the size of an edge
#[derive(Debug, Clone, Copy, new, getset::CopyGetters, Serialize, Deserialize)]
pub struct Size {
    /// The radius of the corridor
    #[getset(get_copy = "pub")]
    radius: f64,
}

/// A direction across an edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// A direction starting from [`Id::from`] and ending at [`Id::to`]
    FromTo,
    /// A direction starting from [`Id::to`] and ending at [`Id::from`]
    ToFrom,
}

/// A position on the cross section of an edge.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CrossSectionPosition(nalgebra::Vector2<f64>);

impl CrossSectionPosition {
    /// Create a new position from the two components.
    pub fn new(x: f64, y: f64) -> Self {
        Self(nalgebra::Vector2::new(x, y))
    }

    /// The vector from the center to the position.
    pub fn vector(self) -> nalgebra::Vector2<f64> {
        self.0
    }

    /// The X-coordinate of [`Self::vector`].
    pub fn x(self) -> f64 {
        self.0.x
    }

    /// The Y-coordinate of [`Self::vector`].
    pub fn y(self) -> f64 {
        self.0.y
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
    /// The type of duct.
    #[getset(get_copy = "pub")]
    ty: DuctType,
    /// The entity storing the duct attributes.
    #[getset(get_copy = "pub")]
    entity: Entity,
}

/// The type of a duct.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuctType {
    /// A rail that vehicles can move along.
    ///
    /// The first parameter is the direction of the rail,
    /// or [`None`] if the rail is disabled.
    Rail(Option<Direction>),
    /// A pipe that liquids can be transferred through.
    ///
    /// The first parameter is the direction of the pipe,
    /// or [`None`] if the rail is disabled.
    ///
    /// The second and third parameters are the liquid storage IDs in the endpoints.
    /// They refer to the "from" and "to" IDs, and do not change when direction is flipped.
    Liquid {
        /// The direction that the pipe runs in
        dir: Option<Direction>,
        /// The storage ordinal in the "from" node.
        ///
        /// This value does **not** swap with [`to_storage`] when the direction is flipped.
        from_storage: usize,
        /// The storage ordinal in the "to" node.
        ///
        /// This value does **not** swap with [`from_storage`] when the direction is flipped.
        to_storage: usize,
    },
    /// A cable that electricity can pass through.
    ///
    /// The first parameter specifies whether the cable is enabled.
    Electricity(bool),
}

impl DuctType {
    /// Whether the duct is active.
    pub fn active(self) -> bool {
        match self {
            Self::Rail(option) => option.is_some(),
            Self::Liquid { dir, .. } => dir.is_some(),
            Self::Electricity(enabled) => enabled,
        }
    }

    /// The direction of the duct, if any.
    pub fn direction(self) -> Option<Direction> {
        match self {
            Self::Rail(dir) | Self::Liquid { dir, .. } => dir,
            _ => None,
        }
    }

    /// Creates a duct entity for this duct type.
    fn create_entity(
        self,
        entities: &mut CommandBuffer,
        world: &SubWorld,
        from: Entity,
        to: Entity,
        radius: f64,
    ) -> Entity {
        use legion::EntityStore;

        let from_entry = world.entry_ref(from).expect("The from node entity does not exist");
        let to_entry = world.entry_ref(to).expect("The to node entity does not exist");

        let from_pos = from_entry
            .get_component::<Position>()
            .expect("The from node entity does not have a position");
        let to_pos = to_entry
            .get_component::<Position>()
            .expect("The to node entity does not have a position");

        let dist = (*to_pos - *from_pos).norm();

        match self {
            DuctType::Electricity(true) => {
                entities.push(()) // TODO
            }
            DuctType::Rail(Some(direction)) => {
                entities.push(()) // TODO
            }
            DuctType::Liquid { dir: Some(direction), from_storage, to_storage } => entities.push({
                let from_list = from_entry
                    .get_component::<liquid::StorageList>()
                    .expect("The from node entity does not have liquid::StorageList");
                let from_tank = *from_list
                    .storages()
                    .get(from_storage)
                    .expect("Pipe definition references a nonexistent from-storage");

                let to_list = to_entry
                    .get_component::<liquid::StorageList>()
                    .expect("The to node entity does not have liquid::StorageList");
                let to_tank = *to_list
                    .storages()
                    .get(to_storage)
                    .expect("Pipe definition references a nonexistent to-storage");

                let (src, dest) = match direction {
                    Direction::FromTo => (from_tank, to_tank),
                    Direction::ToFrom => (to_tank, from_tank),
                };

                let resistance = dist / radius.powi(2);

                (
                    liquid::Pipe::new(src, dest),
                    liquid::PipeResistance::new(resistance),
                    liquid::PipeFlow::default(),
                )
            }),
            DuctType::Electricity(false)
            | DuctType::Rail(None)
            | DuctType::Liquid { dir: None, .. } => {
                entities.push(()) // dummy entity
            }
        }
    }
}

/// Indicates that an edge is added.
#[derive(Debug, new, getset::Getters)]
pub struct AddEvent {
    /// The added edge ID.
    #[getset(get = "pub")]
    edge: Id,
    /// The added edge entity.
    #[getset(get = "pub")]
    entity: Entity,
}

/// Indicates that an edge is flagged for removal
#[derive(Debug, new, getset::Getters)]
pub struct RemoveEvent {
    /// The removed edge
    #[getset(get = "pub")]
    edge: Id,
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
        rot.transpose().prepend_translation(&-from.vector()).append_nonuniform_scaling(
            &Vector::new(1. / size.radius(), 1. / size.radius(), 1. / dir.norm()),
        )
    }
}

/// An event to schedule requests to initialize new edges.
#[derive(TypedBuilder)]
pub struct CreateRequest {
    from: Entity,
    to: Entity,
    size: f64,
    hp: units::Portion<units::Hitpoint>,
    design: Vec<save::SavedDuct>,
}

#[codegen::system]
#[read_component(Position)]
#[read_component(liquid::StorageList)]
fn create_new_edge(
    entities: &mut CommandBuffer,
    world: &SubWorld,
    #[subscriber] requests: impl Iterator<Item = CreateRequest>,
    #[publisher] add_events: impl FnMut(AddEvent),
) {
    for request in requests {
        let design = request
            .design
            .iter()
            .map(|duct| Duct {
                center: duct.center,
                radius: duct.radius,
                ty: duct.ty,
                entity: duct.ty.create_entity(
                    entities,
                    world,
                    request.from,
                    request.to,
                    duct.radius,
                ),
            })
            .collect();
        let id = Id::new(request.from, request.to);
        let entity = entities.push((id, Size::new(request.size), request.hp, Design::new(design)));
        add_events(AddEvent { edge: id, entity });
    }
}

/// An event to schedule requests to initialize saved edges.
#[derive(TypedBuilder)]
pub struct LoadRequest {
    /// The saved edge.
    save: Box<save::Edge>,
}

#[codegen::system]
#[read_component(Position)]
#[read_component(liquid::StorageList)]
fn create_saved_edge(
    entities: &mut CommandBuffer,
    world: &SubWorld,
    #[subscriber] requests: impl Iterator<Item = LoadRequest>,
    #[publisher] add_events: impl FnMut(AddEvent),
    #[resource] index: &node::Index,
) {
    for LoadRequest { save } in requests {
        let from = index.get(save.from).expect("Edge references nonexistent node");
        let to = index.get(save.to).expect("Edge references nonexistent node");
        // FIXME how do we handle the error here properly?

        let design = save
            .design
            .iter()
            .map(|duct| Duct {
                center: duct.center,
                radius: duct.radius,
                ty: duct.ty,
                entity: duct.ty.create_entity(entities, world, from, to, duct.radius),
            })
            .collect();

        let id = Id::new(from, to);
        let entity = entities.push((id, save.size, save.hitpoint, Design::new(design)));
        add_events(AddEvent { edge: id, entity });
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(create_new_edge_setup).uses(create_saved_edge_setup)
}

/// Save type for edges.
pub mod save {
    use super::*;
    use crate::node;

    /// Saves all data related to an edge.
    #[derive(Clone, Serialize, Deserialize)]
    pub struct Edge {
        pub(crate) from: node::Id,
        pub(crate) to: node::Id,
        pub(crate) size: super::Size,
        pub(crate) design: Vec<SavedDuct>,
        pub(crate) hitpoint: units::Portion<units::Hitpoint>,
    }

    /// Saves all data related to a duct.
    #[derive(Clone, Serialize, Deserialize)]
    pub struct SavedDuct {
        /// Center position of the duct.
        pub center: CrossSectionPosition,
        /// Radius of the duct.
        pub radius: f64,
        /// Type of the duct.
        pub ty: DuctType,
    }
}
