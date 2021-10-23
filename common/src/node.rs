//! Node management.
//!
//! A node is an instance of a building.

use std::collections::BTreeMap;
use std::num::NonZeroUsize;

use derive_new::new;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::{Entity, EntityStore};
use smallvec::{smallvec, SmallVec};
use traffloat_def::state;
use typed_builder::TypedBuilder;

use crate::def::feature::Feature;
pub use crate::def::state::NodeId as Id;
use crate::def::{building, CustomizableName};
use crate::space::{Matrix, Position};
use crate::sun::LightStats;
use crate::{appearance, cargo, defense, gas, liquid, population, save, units, vehicle, SetupEcs};

codegen::component_depends! {
    Id = (
        LightStats,
        cargo::StorageList,
        cargo::StorageCapacity,
        liquid::StorageList,
        gas::StorageList,
        gas::StorageCapacity,
        population::StorageList,
    ) + ?(
        defense::Core,
        population::Housing, // TODO multiple housing provisions?
        vehicle::RailPump,
        liquid::Pump,
        gas::Pump,
    )
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
    node:   Id,
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
}

impl Index {
    /// Retrieves the entity ID for the given node
    pub fn get(&self, id: Id) -> Option<Entity> { self.index.get(&id).copied() }
}

#[codegen::system(Command)]
fn delete_nodes(
    cmd_buf: &mut legion::systems::CommandBuffer,
    #[resource] index: &mut Index,
    #[subscriber] removals: impl Iterator<Item = RemoveEvent>,
    #[publisher] post_remove_pub: impl FnMut(PostRemoveEvent),
) {
    let mut count = 0_usize;

    // queue deletion requests for the next event loop
    for removal in removals {
        let entity = index.index.remove(&removal.node).expect("Removing nonexistent node entity");
        cmd_buf.remove(entity);
        count += 1;
    }

    if let Some(count) = NonZeroUsize::new(count) {
        post_remove_pub(PostRemoveEvent { count });
    }
}

/// An event to schedule requests to initialize new nodes.
///
/// Do not subscribe to this event for listening to node creation.
/// Use [`AddEvent`] instead.
#[derive(TypedBuilder)]
pub struct CreationRequest {
    /// Type ID of the building to create.
    type_id:  building::Id,
    /// The position of the node.
    position: Position,
    /// The rotation matrix of the node.
    rotation: Matrix,
}

#[codegen::system(Command)]
fn create_new_node(
    entities: &mut CommandBuffer,
    #[subscriber] requests: impl Iterator<Item = CreationRequest>,
    #[publisher] add_events: impl FnMut(AddEvent),
    #[resource(no_init)] def: &save::GameDefinition,
    #[resource] index: &mut Index,
) {
    for request in requests {
        let building = &def[request.type_id];

        let id = loop {
            let id = Id::new(rand::random());
            if !index.index.contains_key(&id) {
                break id;
            }
        };

        let arbitrary_liquid_type = def.liquid_recipes().default();
        let liquids: SmallVec<_> = building
            .storage()
            .liquid()
            .iter()
            .map(|storage| {
                entities.push((
                    liquid::Storage::new(arbitrary_liquid_type),
                    liquid::NextStorageType::new(arbitrary_liquid_type),
                    liquid::StorageCapacity::new(storage.capacity()),
                    liquid::StorageSize::new(units::LiquidVolume(0.)),
                    liquid::NextStorageSize::new(units::LiquidVolume(0.)),
                    CustomizableName::Original(storage.name().clone()),
                ))
            })
            .collect();

        let entity = entities.push((
            id,
            request.type_id,
            CustomizableName::Original(building.name().clone()),
            request.position,
            appearance::Appearance::new(
                building
                    .shapes()
                    .iter()
                    .map(|shape| {
                        appearance::Component::builder()
                            .unit(shape.unit())
                            .matrix(request.rotation * shape.transform().0)
                            .texture(shape.texture())
                            .build()
                    })
                    .collect(),
            ),
            units::Portion::full(building.hitpoint()),
            LightStats::default(),
            cargo::StorageList::new(smallvec![]),
            cargo::StorageCapacity::new(building.storage().cargo()),
            liquid::StorageList::new(liquids.clone()),
            gas::StorageList::new(smallvec![]),
            gas::StorageCapacity::new(building.storage().gas()),
        ));

        for liquid in liquids {
            entities.add_component(liquid, Child::new(entity));
        }

        init_features(entities, entity, building.features());

        index.index.insert(id, entity);

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
    save: Box<state::Node>,
}

#[codegen::system(Command)]
fn create_saved_node(
    entities: &mut CommandBuffer,
    #[subscriber] requests: impl Iterator<Item = LoadRequest>,
    #[publisher] add_events: impl FnMut(AddEvent),
    #[resource(no_init)] def: &save::GameDefinition,
    #[resource] index: &mut Index,
) {
    for LoadRequest { save } in requests {
        let building = &def[save.building()];

        let cargo_list = save
            .cargo()
            .iter()
            .map(|&(id, size)| {
                let entity = entities.push((
                    cargo::Storage::new(id),
                    cargo::StorageSize::new(size),
                    cargo::NextStorageSize::new(size),
                ));
                (id, entity)
            })
            .collect();

        let liquid_storages = building.storage().liquid();
        let liquid_list: SmallVec<_> = save
            .liquid()
            .iter()
            .zip(liquid_storages.iter())
            .map(|(&(id, size), storage_def)| {
                entities.push((
                    liquid::Storage::new(id),
                    liquid::NextStorageType::new(id),
                    liquid::StorageCapacity::new(storage_def.capacity()),
                    liquid::StorageSize::new(size),
                    liquid::NextStorageSize::new(size),
                    storage_def.name().clone(),
                ))
            })
            .collect();
        let gas_list = save
            .gas()
            .iter()
            .map(|&(id, size)| {
                let entity = entities.push((
                    gas::Storage::new(id),
                    gas::StorageSize::new(size),
                    gas::NextStorageSize::new(size),
                ));
                (id, entity)
            })
            .collect();

        let entity = entities.push((
            save.id(),
            save.building(),
            save.name().clone(),
            save.position(),
            save.appearance().clone(),
            units::Portion::new(save.hitpoint(), building.hitpoint()),
            LightStats::default(),
            cargo::StorageList::new(cargo_list),
            cargo::StorageCapacity::new(building.storage().cargo()),
            liquid::StorageList::new(liquid_list.clone()),
            gas::StorageList::new(gas_list),
            gas::StorageCapacity::new(building.storage().gas()),
        ));

        for liquid in liquid_list {
            entities.add_component(liquid, Child::new(entity));
        }

        init_features(entities, entity, building.features());

        index.index.insert(save.id(), entity);
        add_events(AddEvent { node: save.id(), entity })
    }
}

fn init_features(entities: &mut CommandBuffer, entity: Entity, features: &[Feature]) {
    for feature in features {
        match feature {
            Feature::Core => entities.add_component(entity, defense::Core),
            Feature::ProvidesHousing(housing) => {
                entities.add_component(entity, population::Housing::new(housing.storage()))
            }
            Feature::Reaction(_) => todo!(),
            Feature::RailPump(_) => todo!("refactor to entity list"),
            Feature::LiquidPump(_) => todo!("refactor to entity list"),
            Feature::GasPump(_) => todo!("refactor to entity list"),
            Feature::SecureEntry(_) => todo!(),
            Feature::SecureExit(_) => todo!(),
        }
    }
}

#[codegen::system(Visualize)]
#[read_component(Id)]
#[read_component(building::Id)]
#[read_component(CustomizableName)]
#[read_component(Position)]
#[read_component(appearance::Appearance)]
#[read_component(units::Portion<units::Hitpoint>)]
#[read_component(cargo::StorageList)]
#[read_component(gas::StorageList)]
#[read_component(liquid::StorageList)]
fn save_nodes(world: &mut SubWorld, #[subscriber] requests: impl Iterator<Item = save::Request>) {
    use legion::IntoQuery;

    let mut query = <(
        &Id,
        &building::Id,
        &CustomizableName,
        &Position,
        &appearance::Appearance,
        &units::Portion<units::Hitpoint>,
        &cargo::StorageList,
        &gas::StorageList,
        &liquid::StorageList,
    )>::query();

    let (query_world, ra_world) = world.split_for_query(&query);

    for request in requests {
        for (
            &id,
            &building,
            name,
            &position,
            appearance,
            hitpoint,
            cargo_list,
            gas_list,
            liquid_list,
        ) in query.iter(&query_world)
        {
            let cargo = cargo_list
                .storages()
                .iter()
                .map(|&(cargo_ty, storage)| {
                    let storage = ra_world.entry_ref(storage).expect("Dangling entity reference");
                    // TODO confirm that the deserialized state will setup NextStorageSize correctly.
                    let size = storage
                        .get_component::<cargo::StorageSize>()
                        .expect("Malformed entity reference");
                    (cargo_ty, size.size())
                })
                .collect();
            let gas = gas_list
                .storages()
                .iter()
                .map(|&(gas_ty, storage)| {
                    let storage = ra_world.entry_ref(storage).expect("Dangling entity reference");
                    // TODO confirm that the deserialized state will setup NextStorageSize correctly.
                    let size = storage
                        .get_component::<gas::StorageSize>()
                        .expect("Malformed entity reference");
                    (gas_ty, size.size())
                })
                .collect();
            let liquid = liquid_list
                .storages()
                .iter()
                .map(|&storage| {
                    let storage = ra_world.entry_ref(storage).expect("Dangling entity reference");
                    // TODO confirm that the deserialized state will setup NextStorageType and NextStorageSize correctly.
                    let liquid_ty = storage
                        .get_component::<liquid::Storage>()
                        .expect("Malformed entity reference")
                        .liquid();
                    let size = storage
                        .get_component::<liquid::StorageSize>()
                        .expect("Malformed entity reference");
                    (liquid_ty, size.size())
                })
                .collect();

            let node = state::Node::builder()
                .id(id)
                .building(building)
                .name(name.clone())
                .position(position)
                .appearance(appearance.clone())
                .hitpoint(hitpoint.current())
                .cargo(cargo)
                .gas(gas)
                .liquid(liquid)
                .build();

            {
                let mut file = request.file();
                file.state_mut().nodes_mut().push(node);
            }
        }
    }
}

/// Initializes ECS
pub fn setup_ecs(setup: SetupEcs) -> SetupEcs {
    setup
        .uses(delete_nodes_setup)
        .uses(create_new_node_setup)
        .uses(create_saved_node_setup)
        .uses(save_nodes_setup)
}
