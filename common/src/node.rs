//! Node management.
//!
//! A node is an instance of a building.

use std::collections::BTreeMap;
use std::num::NonZeroUsize;

use arcstr::ArcStr;
use derive_new::new;
use legion::{systems::CommandBuffer, Entity};
use serde::{Deserialize, Serialize};
use smallvec::{smallvec, SmallVec};
use typed_builder::TypedBuilder;

use crate::def::{building, GameDefinition};
use crate::defense;
use crate::shape::{self, Shape};
use crate::space::{Matrix, Position};
use crate::sun::LightStats;
use crate::units;
use crate::SetupEcs;
use crate::{cargo, gas, liquid};
use crate::{population, vehicle};

/// Component storing an identifier for a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, new, Serialize, Deserialize)]
pub struct Id {
    inner: u32,
}

codegen::component_depends! {
    Id = (
        Id,
        Name,
        Position,
        Shape,
        LightStats,
        units::Portion<units::Hitpoint>,
        cargo::StorageList,
        cargo::StorageCapacity,
        liquid::StorageList,
        gas::StorageList,
        gas::StorageCapacity,
    ) + ?(
        defense::Core,
        population::Housing,
        vehicle::RailPump,
        liquid::Pump,
        gas::Pump,
    )
}

/// Component storing the name of the node
#[derive(Debug, Clone, new, getset::Getters, getset::Setters, Serialize, Deserialize)]
pub struct Name {
    /// Name of the node
    #[getset(get = "pub")]
    #[getset(set = "pub")]
    name: ArcStr,
}

/// A component applied to child entities of a node.
#[derive(Debug, Clone, new, getset::CopyGetters)]
pub struct Child {
    /// The entity ID of the parent node entity.
    #[getset(get_copy = "pub")]
    parent: Entity,
}

/// Indicates that a node is added
#[derive(Debug, new, getset::CopyGetters)]
pub struct AddEvent {
    /// The added node ID
    #[getset(get_copy = "pub")]
    node: Id,
    /// The added node entity
    #[getset(get_copy = "pub")]
    entity: Entity,
}

/// Indicates that a node is flagged for removal
#[derive(Debug, new, getset::CopyGetters)]
pub struct RemoveEvent {
    /// The removed node
    #[getset(get_copy = "pub")]
    node: Id,
}

/// Indicates that nodes have been removed
#[derive(Debug, new, getset::CopyGetters)]
pub struct PostRemoveEvent {
    /// Number of nodes removed
    #[getset(get_copy = "pub")]
    count: NonZeroUsize,
}

/// Tracks the nodes in the world
#[derive(Default)]
pub struct Index {
    index: BTreeMap<Id, Entity>,
    deletion_queue: Vec<Id>,
}

impl Index {
    /// Retrieves the entity ID for the given node
    pub fn get(&self, id: Id) -> Option<Entity> {
        self.index.get(&id).copied()
    }
}

#[codegen::system]
fn delete_nodes(
    cmd_buf: &mut legion::systems::CommandBuffer,
    #[resource] index: &mut Index,
    #[subscriber] removals: impl Iterator<Item = RemoveEvent>,
    #[publisher] post_remove_pub: impl FnMut(PostRemoveEvent),
) {
    for &node in &index.deletion_queue {
        let entity = index
            .index
            .remove(&node)
            .expect("Removing nonexistent node entity");
        cmd_buf.remove(entity);
    }
    let count = index.deletion_queue.len();
    index.deletion_queue.clear();
    if let Some(count) = NonZeroUsize::new(count) {
        post_remove_pub(PostRemoveEvent { count });
    }

    // queue deletion requests for the next event loop
    for removal in removals {
        index.deletion_queue.push(removal.node);
    }
}

/// An event to schedule requests to initialize new nodes.
///
/// Do not subscribe to this event for listening to node creation.
/// Use [`AddEvent`] instead.
#[derive(TypedBuilder)]
pub struct CreationRequest {
    /// Type ID of the building to create.
    type_id: building::TypeId,
    /// The position of the node.
    position: Position,
    /// The rotation matrix of the node.
    rotation: Matrix,
}

#[codegen::system]
fn create_new_node(
    entities: &mut CommandBuffer,
    #[subscriber] requests: impl Iterator<Item = CreationRequest>,
    #[publisher] add_events: impl FnMut(AddEvent),
    #[resource(no_init)] def: &GameDefinition,
    #[resource] index: &mut Index,
) {
    for request in requests {
        let building = def
            .building()
            .get(&request.type_id)
            .expect("Received invalid type ID");

        let id = Id::new(rand::random());

        let arbitrary_liquid_type = def.liquid().first().expect("at least one liquid type").0;
        let liquids: SmallVec<_> = building
            .storage()
            .liquid()
            .iter()
            .map(|&volume| {
                entities.push((
                    liquid::Storage::new(arbitrary_liquid_type.clone()),
                    liquid::NextStorageType::new(arbitrary_liquid_type.clone()),
                    liquid::StorageCapacity::new(volume),
                    liquid::StorageSize::new(units::LiquidVolume(0.)),
                    liquid::NextStorageSize::new(units::LiquidVolume(0.)),
                ))
            })
            .collect();

        let entity = entities.push((
            id,
            Name::new(building.name().clone()),
            request.position,
            Shape::builder()
                .unit(building.shape().unit())
                .matrix(request.rotation * building.shape().transform())
                .texture(shape::Texture::new(
                    building.shape().texture_src().clone(),
                    building.shape().texture_name().clone(),
                ))
                .build(),
            LightStats::default(),
            units::Portion::full(building.hitpoint()),
            cargo::StorageList::new(smallvec![]),
            cargo::StorageCapacity::new(building.storage().cargo()),
            liquid::StorageList::new(liquids.clone()),
            gas::StorageList::new(smallvec![]),
            gas::StorageCapacity::new(building.storage().gas()),
        ));

        for liquid in liquids {
            entities.add_component(liquid, Child::new(entity));
        }

        index.index.insert(id, entity);

        for feature in building.features() {
            match feature {
                building::ExtraFeature::Core => {
                    entities.add_component(entity, defense::Core);
                }
                building::ExtraFeature::ProvidesHousing(housing) => {
                    entities.add_component(
                        entity,
                        population::Housing::builder().capacity(*housing).build(),
                    );
                }
                building::ExtraFeature::RailPump(force) => entities
                    .add_component(entity, vehicle::RailPump::builder().force(*force).build()),
                building::ExtraFeature::LiquidPump(force) => {
                    entities.add_component(entity, liquid::Pump::builder().force(*force).build())
                }
                building::ExtraFeature::GasPump(force) => {
                    entities.add_component(entity, gas::Pump::builder().force(*force).build())
                }
                building::ExtraFeature::SecureEntry {
                    skill,
                    min_level,
                    breach_probability,
                } => {
                    todo!(
                        "Create entity with {:?} {:?} {:?}",
                        skill,
                        min_level,
                        breach_probability
                    )
                }
                building::ExtraFeature::SecureExit {
                    skill,
                    min_level,
                    breach_probability,
                } => {
                    todo!(
                        "Create entity with {:?} {:?} {:?}",
                        skill,
                        min_level,
                        breach_probability
                    )
                }
            }
        }

        add_events(AddEvent { node: id, entity })
    }
}

/// An event to schedule requests to initialize saved nodes.
///
/// Do not subscribe to this event for listening to node creation.
/// Use [`AddEvent`] instead.
#[derive(TypedBuilder)]
pub struct LoadRequest {
    /// The saved node.
    save: Box<save::Node>,
}

#[codegen::system]
fn create_saved_node(
    entities: &mut CommandBuffer,
    #[subscriber] requests: impl Iterator<Item = LoadRequest>,
    #[publisher] add_events: impl FnMut(AddEvent),
    #[resource] index: &mut Index,
) {
    for LoadRequest { save } in requests {
        let cargo_list = save
            .cargo
            .iter()
            .map(|(id, &size)| {
                let entity = entities.push((
                    cargo::Storage::new(id.clone()),
                    cargo::StorageSize::new(size),
                    cargo::NextStorageSize::new(size),
                ));
                (id.clone(), entity)
            })
            .collect();
        let liquid_list: SmallVec<_> = save
            .liquid
            .iter()
            .map(|storage| {
                entities.push((
                    liquid::Storage::new(storage.ty.clone()),
                    liquid::NextStorageType::new(storage.ty.clone()),
                    liquid::StorageCapacity::new(storage.capacity),
                    liquid::StorageSize::new(storage.volume),
                    liquid::NextStorageSize::new(storage.volume),
                ))
            })
            .collect();
        let gas_list = save
            .gas
            .iter()
            .map(|(id, &size)| {
                let entity = entities.push((
                    gas::Storage::new(id.clone()),
                    gas::StorageSize::new(size),
                    gas::NextStorageSize::new(size),
                ));
                (id.clone(), entity)
            })
            .collect();

        let entity = entities.push((
            save.id,
            save.name.clone(),
            save.position,
            save.shape.clone(),
            LightStats::default(),
            save.hitpoint,
            cargo::StorageList::new(cargo_list),
            cargo::StorageCapacity::new(save.cargo_capacity),
            liquid::StorageList::new(liquid_list.clone()),
            gas::StorageList::new(gas_list),
            gas::StorageCapacity::new(save.gas_capacity),
        ));

        for liquid in liquid_list {
            entities.add_component(liquid, Child::new(entity));
        }

        if save.is_core {
            entities.add_component(entity, defense::Core);
        }
        if let Some(housing) = save.housing_provision {
            entities.add_component(
                entity,
                population::Housing::builder().capacity(housing).build(),
            );
        }
        if let Some(force) = save.rail_pump {
            entities.add_component(entity, vehicle::RailPump::builder().force(force).build());
        }
        if let Some(force) = save.liquid_pump {
            entities.add_component(entity, liquid::Pump::builder().force(force).build());
        }
        if let Some(force) = save.gas_pump {
            entities.add_component(entity, gas::Pump::builder().force(force).build());
        }

        index.index.insert(save.id, entity);
        add_events(AddEvent {
            node: save.id,
            entity,
        })
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup
        .uses(delete_nodes_setup)
        .uses(create_new_node_setup)
        .uses(create_saved_node_setup)
}

/// Save type for nodes.
pub mod save {
    use std::collections::BTreeMap;

    use super::*;
    use crate::def;
    use crate::units;

    /// Saves all data related to a node.
    #[derive(Clone, Serialize, Deserialize)]
    pub struct Node {
        pub(crate) id: super::Id,
        pub(crate) name: super::Name,
        pub(crate) position: Position,
        pub(crate) shape: Shape,
        pub(crate) hitpoint: units::Portion<units::Hitpoint>,
        pub(crate) cargo: BTreeMap<def::cargo::TypeId, units::CargoSize>,
        pub(crate) cargo_capacity: units::CargoSize,
        pub(crate) liquid: Vec<LiquidStorage>,
        pub(crate) gas: BTreeMap<def::gas::TypeId, units::GasVolume>,
        pub(crate) gas_capacity: units::GasVolume,
        pub(crate) is_core: bool,
        pub(crate) housing_provision: Option<u32>,
        pub(crate) rail_pump: Option<units::RailForce>,
        pub(crate) liquid_pump: Option<units::PipeForce>,
        pub(crate) gas_pump: Option<units::FanForce>,
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub(crate) struct LiquidStorage {
        pub(crate) ty: def::liquid::TypeId,
        pub(crate) volume: units::LiquidVolume,
        pub(crate) capacity: units::LiquidVolume,
    }
}
