//! Edge management.
//!
//! An edge is also called a "corridor".
//! It connects two nodes together.

use def::building;
use derive_new::new;
use legion::systems::CommandBuffer;
use legion::world::{EntryRef, SubWorld};
use legion::{Entity, EntityStore};
use serde::{Deserialize, Serialize};
pub use traffloat_def::edge::{
    Direction, DuctType, ElectricityDuctType, LiquidDuctType, RailDuctType,
};
use typed_builder::TypedBuilder;

use crate::space::{Matrix, Position, Vector};
use crate::{def, liquid, node, save, units, SetupEcs};

/// Component storing the endpoints of an edge
#[derive(Debug, Clone, Copy, PartialEq, Eq, new, getset::CopyGetters, getset::Setters)]
pub struct Id {
    /// The "alpha" node
    #[getset(get_copy = "pub")]
    alpha: Entity,
    /// The "beta" node
    #[getset(get_copy = "pub")]
    beta:  Entity,
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
    alpha: Entity,
    beta: Entity,
    radius: f64,
) -> Entity {
    let alpha_entry = world.entry_ref(alpha).expect("The alpha node entity does not exist");
    let beta_entry = world.entry_ref(beta).expect("The beta node entity does not exist");

    let alpha_pos = alpha_entry
        .get_component::<Position>()
        .expect("The alpha node entity does not have a position");
    let beta_pos = beta_entry
        .get_component::<Position>()
        .expect("The beta node entity does not have a position");

    let dist = (*beta_pos - *alpha_pos).norm();

    match ty {
        DuctType::Electricity(ty) => {
            #[allow(clippy::if_same_then_else)] // TODO fix
            if ty.disabled() {
                entities.push(()) // create a dummy entity
            } else {
                entities.push(()) // TODO
            }
        }
        DuctType::Rail(ty) => {
            #[allow(clippy::if_same_then_else)] // TODO fix
            if let Some(_dir) = ty.direction() {
                entities.push(()) // TODO
            } else {
                entities.push(()) // create a dummy entity
            }
        }
        DuctType::Liquid(ty) => {
            match ty {
                LiquidDuctType::AlphaToBeta { alpha_storage, beta_storage }
                | LiquidDuctType::BetaToAlpha { beta_storage, alpha_storage } => entities.push({
                    fn get_tank(
                        entry: EntryRef<'_>,
                        storage: building::storage::liquid::Id,
                    ) -> Entity {
                        let list = entry
                            .get_component::<liquid::StorageList>()
                            .expect("Node entity has no liquid::StorageList");
                        let tank = list
                            .storages()
                            .get(storage.index())
                            .expect("Pipe definition references nonexistent storage");
                        *tank
                    }

                    let alpha_tank = get_tank(alpha_entry, alpha_storage);
                    let beta_tank = get_tank(beta_entry, beta_storage);

                    let (src, dest) = match ty {
                        LiquidDuctType::AlphaToBeta { .. } => (alpha_tank, beta_tank),
                        LiquidDuctType::BetaToAlpha { .. } => (beta_tank, alpha_tank),
                        _ => unreachable!("Within match arm"),
                    };

                    let resistance = dist / radius.powi(2);

                    (
                        liquid::Pipe::new(src, dest),
                        liquid::PipeResistance::new(resistance),
                        liquid::PipeFlow::default(),
                    )
                }),
                LiquidDuctType::Disabled => {
                    entities.push(()) // create a dummy entity
                }
            }
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
    let alpha = edge.alpha();
    let beta = edge.beta();

    let alpha: Position = *world
        .entry_ref(alpha)
        .expect("alpha_entity does not exist")
        .get_component()
        .expect("alpha node does not have Position");
    let beta: Position = *world
        .entry_ref(beta)
        .expect("beta_entity does not exist")
        .get_component()
        .expect("beta node does not have Position");

    let dir = beta - alpha;
    let rot = match nalgebra::Rotation3::rotation_between(&Vector::new(0., 0., 1.), &dir) {
        Some(rot) => rot.to_homogeneous(),
        None => Matrix::identity().append_nonuniform_scaling(&Vector::new(0., 0., -1.)),
    };

    if from_unit {
        rot.prepend_nonuniform_scaling(&Vector::new(size.radius(), size.radius(), dir.norm()))
            .append_translation(&alpha.vector())
    } else {
        rot.transpose().prepend_translation(&-alpha.vector()).append_nonuniform_scaling(
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
    save: Box<def::edge::Edge>,
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
        let alpha = index.get(save.endpoints().alpha()).expect("Edge references nonexistent node");
        let beta = index.get(save.endpoints().beta()).expect("Edge references nonexistent node");
        // FIXME how do we handle the error here properly?

        let design = save
            .ducts()
            .iter()
            .map(|duct| {
                Duct::builder()
                    .center(CrossSectionPosition::new(duct.center().x, duct.center().y))
                    .radius(duct.radius())
                    .ty(duct.ty())
                    .entity(create_duct(duct.ty(), entities, world, alpha, beta, duct.radius()))
                    .build()
            })
            .collect();

        let id = Id::new(alpha, beta);
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
            let alpha = get_node_id(id.alpha());
            let beta = get_node_id(id.beta());

            let edge = def::edge::Edge::builder()
                .endpoints(def::edge::AlphaBeta::new(alpha, beta))
                .radius(size.radius())
                .hitpoint(hitpoint)
                .ducts(
                    design
                        .ducts()
                        .iter()
                        .map(|duct| {
                            def::edge::Duct::builder()
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
