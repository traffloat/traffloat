//! Edge management.
//!
//! An edge is also called a "corridor".
//! It connects two nodes together.

use derive_new::new;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::{Entity, EntityStore};
use serde::{Deserialize, Serialize};
use traffloat_def::state;
pub use traffloat_def::state::{Direction, DuctType};
use typed_builder::TypedBuilder;

use crate::space::{Matrix, Position, Vector};
use crate::{liquid, node, save, units, SetupEcs};

/// Component storing the endpoints of an edge
#[derive(Debug, Clone, Copy, PartialEq, Eq, new, getset::CopyGetters, getset::Setters)]
pub struct Id {
    /// The "source" node
    #[getset(get_copy = "pub")]
    from: Entity,
    /// The "dest" node
    #[getset(get_copy = "pub")]
    to:   Entity,
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

/// A position on the cross section of an edge.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CrossSectionPosition(nalgebra::Vector2<f64>);

impl CrossSectionPosition {
    /// Create a new position from the two components.
    pub fn new(x: f64, y: f64) -> Self { Self(nalgebra::Vector2::new(x, y)) }

    /// The vector from the center to the position.
    pub fn vector(self) -> nalgebra::Vector2<f64> { self.0 }

    /// The X-coordinate of [`Self::vector`].
    pub fn x(self) -> f64 { self.0.x }

    /// The Y-coordinate of [`Self::vector`].
    pub fn y(self) -> f64 { self.0.y }
}

/// The geometric design of the edge.
///
/// This is only used during graphical user interaction and serialization.
/// Simulation systems should depend on the actual duct entities
/// instead of computing flow rates from this data structure.
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
    ty:     DuctType,
    /// The entity storing the duct attributes.
    #[getset(get_copy = "pub")]
    entity: Entity,
}

/// Creates a duct entity for a duct type.
fn create_duct(
    ty: DuctType,
    entities: &mut CommandBuffer,
    world: &SubWorld,
    from: Entity,
    to: Entity,
    radius: f64,
) -> Entity {
    let from_entry = world.entry_ref(from).expect("The from node entity does not exist");
    let to_entry = world.entry_ref(to).expect("The to node entity does not exist");

    let from_pos = from_entry
        .get_component::<Position>()
        .expect("The from node entity does not have a position");
    let to_pos =
        to_entry.get_component::<Position>().expect("The to node entity does not have a position");

    let dist = (*to_pos - *from_pos).norm();

    match ty {
        DuctType::Electricity(true) => {
            entities.push(()) // TODO
        }
        DuctType::Rail(Some(_direction)) => {
            entities.push(()) // TODO
        }
        DuctType::Liquid {
            dir: Some(direction),
            src_storage: from_storage,
            dest_storage: to_storage,
        } => entities.push({
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
                Direction::From2To => (from_tank, to_tank),
                Direction::To2From => (to_tank, from_tank),
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

/// Indicates that an edge is added.
#[derive(Debug, new, getset::Getters)]
pub struct AddEvent {
    /// The added edge ID.
    #[getset(get = "pub")]
    edge:   Id,
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
    to:   Entity,
    size: f64,
    hp:   units::Portion<units::Hitpoint>,
}

#[codegen::system(CreateChild)]
fn create_new_edge(
    entities: &mut CommandBuffer,
    #[subscriber] requests: impl Iterator<Item = CreateRequest>,
    #[publisher] add_events: impl FnMut(AddEvent),
) {
    for request in requests {
        let design = Vec::new();
        let id = Id::new(request.from, request.to);
        let entity = entities.push((id, Size::new(request.size), request.hp, Design::new(design)));
        add_events(AddEvent { edge: id, entity });
    }
}

/// An event to schedule requests to initialize saved edges.
#[derive(TypedBuilder)]
pub struct LoadRequest {
    /// The saved edge.
    save: Box<state::Edge>,
}

#[codegen::system(CreateChild)]
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
        let from = index.get(save.from()).expect("Edge references nonexistent node");
        let to = index.get(save.to()).expect("Edge references nonexistent node");
        // FIXME how do we handle the error here properly?

        let design = save
            .design()
            .iter()
            .map(|duct| {
                Duct::builder()
                    .center(CrossSectionPosition::new(duct.center().x, duct.center().y))
                    .radius(duct.radius())
                    .ty(duct.ty())
                    .entity(create_duct(duct.ty(), entities, world, from, to, duct.radius()))
                    .build()
            })
            .collect();

        let id = Id::new(from, to);
        let entity =
            entities.push((id, Size::new(save.radius()), save.hitpoint(), Design::new(design)));
        add_events(AddEvent { edge: id, entity });
    }
}

#[codegen::system(Visualize)]
#[read_component(Id)]
#[read_component(Size)]
#[read_component(units::Portion<units::Hitpoint>)]
#[read_component(Design)]
#[read_component(node::Id)]
fn save_edges(world: &mut SubWorld, #[subscriber] requests: impl Iterator<Item = save::Request>) {
    use legion::IntoQuery;

    for request in requests {
        let mut query = <(&Id, &Size, &units::Portion<units::Hitpoint>, &Design)>::query();
        let (query_world, ra_world) = world.split_for_query(&query);

        for (id, size, &hitpoint, design) in query.iter(&query_world) {
            let get_node_id = |entity| {
                let entry = ra_world.entry_ref(entity).expect("Dangling edge endpoint");
                *entry.get_component::<node::Id>().expect("Edge endpoint is not a node")
            };
            let from = get_node_id(id.from());
            let to = get_node_id(id.to());

            let edge = state::Edge::builder()
                .from(from)
                .to(to)
                .radius(size.radius())
                .hitpoint(hitpoint)
                .design(
                    design
                        .ducts()
                        .iter()
                        .map(|duct| {
                            state::Duct::builder()
                                .center(duct.center().vector())
                                .radius(duct.radius())
                                .ty(duct.ty())
                                .build()
                        })
                        .collect(),
                )
                .build();

            {
                let mut file = request.file();
                file.state_mut().edges_mut().push(edge);
            }
        }
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup.uses(create_new_edge_setup).uses(create_saved_edge_setup).uses(save_edges_setup)
}
