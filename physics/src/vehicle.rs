use std::collections::VecDeque;

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::name::Name;
use bevy::ecs::query::{QueryData, With, Without};
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{EntityCommand, Query};
use bevy::ecs::world::{EntityWorldMut, World};
use bevy::math::Vec3;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

pub mod def;
pub use def::Def as TypeDef;
mod motion;
mod persist;
pub use persist::Persist;
mod persist_type;
pub use persist_type::Persist as PersistTypes;
pub mod propulsion;
pub use propulsion::Propulsion;
mod rail;
pub use rail::*;
use traffloat_proto::proto::{self, AlphaOrBeta};

use crate::graph::conduit;
use crate::util::{QueryExt, Which, WorldExt};
use crate::{fluid, view};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Types>();
        app.register_type::<NextVehicleId>();
        app.register_type::<Vehicle>();
        app.register_type::<Location>();
        app.register_type::<ListInBuilding>();
        app.register_type::<ListOnRail>();
        app.register_type::<CompartmentList>();
        app.register_type::<CompartmentOf>();
        app.register_type::<CompartmentHasVent>();
        app.register_type::<CompartmentVentOf>();
        app.register_type::<CompartmentPassengerList>();
        app.register_type::<PassengerOfCompartment>();
        app.register_type::<OperatorList>();
        app.register_type::<OperatorOf>();

        app.init_resource::<Types>();
        app.init_resource::<Conf>();
        app.init_resource::<NextVehicleId>();

        app.add_plugins(motion::Plug);
        app.add_plugins(propulsion::Plug);

        app.add_systems(
            app::Update,
            init_viewer_system
                .in_set(view::SendUpdatesSystemSet::Init)
                .in_set(view::InitSystemSets::Vehicle),
        );
        app.add_systems(
            app::Update,
            incr_viewer_system
                .in_set(view::SendUpdatesSystemSet::Incr)
                .in_set(view::IncrSystemSets::Vehicle),
        );
    }
}

/// Identifies a vehicle type, indexes [`Types::types`].
///
/// Unlike [`Entity`], this is a stable identifier that is exactly restored
/// across network sync and persistence.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Reflect,
)]
pub struct TypeId(pub u32);

#[derive(Debug, Clone, Default, Serialize, Deserialize, Reflect, Resource)]
pub struct Types {
    types: Vec<TypeDef>,
}

impl Types {
    #[must_use]
    pub fn get(&self, id: TypeId) -> &TypeDef {
        self.types.get(id.0 as usize).expect("got invalid vehicle type reference")
    }

    pub fn push(&mut self, def: TypeDef) -> TypeId {
        let id = u32::try_from(self.types.len()).expect("too many vehicle types");
        self.types.push(def);
        TypeId(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (TypeId, &TypeDef)> {
        self.types
            .iter()
            .enumerate()
            .map(|(i, def)| (TypeId(u32::try_from(i).expect("too many vehicle types")), def))
    }

    fn cleanup_hook(world: &mut World) { world.resource_mut::<Types>().types.clear(); }
}

#[derive(Resource, Reflect, Default)]
struct NextVehicleId(u64);

#[derive(Component, Reflect)]
#[require(propulsion::Status)]
pub struct Vehicle {
    pub ty:   TypeId,
    pub mass: f32,
}

#[derive(Debug, Clone, Copy, Component, Reflect)]
pub enum Location {
    Building { building: Entity, interior_pos: Vec3, speed: Vec3 },
    Rail { conduit: Entity, distance_from_alpha: f32, speed_from_alpha: f32 },
}

/// Component on buildings, referencing the vehicle entities.
#[derive(Component, Reflect)]
pub struct ListInBuilding(Vec<Entity>);

/// Component on rail conduit.
#[derive(Component, Reflect)]
pub struct ListOnRail {
    /// List of vehicle entities on the rail, ordered by distance from alpha.
    deque: VecDeque<Entity>,
}

/// Component on vehicles, referencing the compartment entities.
#[derive(Component, Reflect)]
#[relationship_target(relationship = CompartmentOf, linked_spawn)]
pub struct CompartmentList(Vec<Entity>);

impl CompartmentList {
    pub fn nth(&self, n: usize) -> Option<Entity> { self.0.get(n).copied() }
}

/// Component on compartments, referencing the vehicle entity.
#[derive(Component, Reflect)]
#[relationship(relationship_target = CompartmentList)]
pub struct CompartmentOf(pub Entity);

/// Component on compartments, referencing the passenger resident entities.
#[derive(Component, Reflect)]
#[relationship_target(relationship = PassengerOfCompartment)]
pub struct CompartmentPassengerList(Vec<Entity>);

/// Component on residents, referencing the compartment entity.
#[derive(Component, Reflect)]
#[relationship(relationship_target = CompartmentPassengerList)]
pub struct PassengerOfCompartment {
    #[relationship]
    pub compartment:       Entity,
    pub compartment_index: usize,
}

/// Component on vehicles, referencing the operator resident entities.
#[derive(Component, Reflect)]
#[relationship_target(relationship = OperatorOf)]
pub struct OperatorList(Vec<Entity>);

/// Component on residents, referencing the vehicle entity.
#[derive(Component, Reflect)]
#[relationship(relationship_target = OperatorList)]
pub struct OperatorOf {
    #[relationship]
    pub vehicle: Entity,
    /// Index of the operator slot.
    pub slot:    usize,
}

#[derive(Component, Reflect)]
#[relationship_target(relationship = CompartmentVentOf)]
pub struct CompartmentHasVent(Entity);

#[derive(Component, Reflect)]
#[relationship(relationship_target = CompartmentHasVent)]
pub struct CompartmentVentOf(pub Entity);

#[derive(Resource, Reflect)]
pub struct Conf {
    /// Distance per second within a building without propulsion.
    pub standard_drifting_speed: f32,
}

impl Default for Conf {
    fn default() -> Self { Self { standard_drifting_speed: 3.0 } }
}

pub struct AddTypeCommand {
    pub def: TypeDef,
}

impl AddTypeCommand {
    pub fn run(self, world: &mut World) -> TypeId {
        let mut types = world.resource_mut::<Types>();
        types.push(self.def)
    }
}

pub struct SpawnCommand {
    pub ty:       TypeId,
    pub location: Location,
    pub name:     Option<String>,
}

impl EntityCommand for SpawnCommand {
    type Out = ();

    fn apply(self, mut entity: EntityWorldMut) {
        let ambient_entity = match self.location {
            Location::Building { building, .. } => building,
            Location::Rail { conduit, .. } => {
                let Some(&conduit::OfCorridor(corridor)) = entity.world().log_get(conduit) else {
                    return;
                };
                corridor
            }
        };

        let name = self.name.unwrap_or_else(|| {
            let mut next_id = entity.resource_mut::<NextVehicleId>();
            let id = next_id.0;
            next_id.0 += 1;
            let def = entity.resource::<Types>().get(self.ty);
            format!("{} #{id}", def.name)
        });

        let def = entity.resource::<Types>().get(self.ty);
        let mass = def.physical.mass;

        let cpmts: Vec<_> = def
            .compartments
            .iter()
            .map(|cpmt| {
                let add_storage_cmd = fluid::AddStorageCommand {
                    volume:         cpmt.volume,
                    optical_length: cpmt.volume.cbrt(),
                };
                let mut add_edge_cmd = fluid::AddEdgeCommand {
                    area:             cpmt.vent_area,
                    resistance_recip: cpmt.vent_resistance_recip,
                    alpha:            Entity::PLACEHOLDER,
                    beta:             ambient_entity,
                };

                ((CompartmentOf(entity.id()),), move |cpmt_entity: &mut EntityWorldMut| {
                    let cmpt_entity_id = cpmt_entity.id();
                    add_edge_cmd.alpha = cmpt_entity_id;

                    cpmt_entity.reborrow_scope(|child| add_storage_cmd.apply(child));
                    cpmt_entity.world_scope(|world| {
                        let vent_entity = world.spawn(CompartmentVentOf(cmpt_entity_id));
                        add_edge_cmd.apply(vent_entity);
                    });
                })
            })
            .collect();

        entity.insert((
            Name::new(format!("Vehicle {name}")),
            view::Named { name: name.clone() },
            Vehicle { ty: self.ty, mass },
        ));
        entity.with_related_entities::<CompartmentOf>(|spawner| {
            for (cpmt, post_spawn) in cpmts {
                let mut child = spawner.spawn(cpmt);
                post_spawn(&mut child);
            }
        });

        entity.reborrow_scope(|entity| view::AddViewableCommand.apply(entity));
        LocationTransitionCommand { new_location: self.location, ab_hint: () }.apply(entity);
    }
}

pub struct LocationTransitionCommand<Ab> {
    pub new_location: Location,
    pub ab_hint:      Ab,
}

pub trait WhichHint: Send + Sync + 'static {
    fn into_option(self) -> Option<AlphaOrBeta>;
}

impl<Ab: Which> WhichHint for Ab {
    fn into_option(self) -> Option<AlphaOrBeta> { Some(self.proto()) }
}

impl WhichHint for () {
    fn into_option(self) -> Option<AlphaOrBeta> { None }
}

impl<Ab: WhichHint> EntityCommand for LocationTransitionCommand<Ab> {
    type Out = ();

    fn apply(self, mut entity: EntityWorldMut) {
        let entity_id = entity.id();

        if let Some(old) = entity.get::<Location>() {
            match *old {
                Location::Building { building, .. } => {
                    entity.world_scope(|world| {
                        if let Some(mut list) = world.log_get_mut::<ListInBuilding>(building) {
                            list.0.retain(|&e| e != entity_id);
                        }
                    });
                }
                Location::Rail { conduit, .. } => {
                    entity.world_scope(|world| {
                        if let Some(mut list) = world.log_get_mut::<ListOnRail>(conduit) {
                            if list.deque.back() == Some(&entity_id) {
                                // retain will scan from front,
                                // but back is much more likely than list.deque[1]
                                list.deque.pop_back();
                            } else {
                                list.deque.retain(|&e| e != entity_id);
                            }
                        }
                    });
                }
            }
        }

        entity.insert(self.new_location);
        let world = entity.into_world_mut();

        match self.new_location {
            Location::Building { building, .. } => {
                match world.get_mut::<ListInBuilding>(building) {
                    Some(mut list) => list.0.push(entity_id),
                    None => {
                        world.entity_mut(building).insert(ListInBuilding([entity_id].into()));
                    }
                }
            }
            Location::Rail { conduit, .. } => match world.get_mut::<ListOnRail>(conduit) {
                Some(mut list) => match self.ab_hint.into_option() {
                    Some(AlphaOrBeta::Alpha) => list.deque.push_front(entity_id),
                    Some(AlphaOrBeta::Beta) => list.deque.push_back(entity_id),
                    None => {
                        list.deque.partition_point(|&e| e < entity_id);
                    }
                },
                None => {
                    world.entity_mut(conduit).insert(ListOnRail { deque: [entity_id].into() });
                }
            },
        }
    }
}

fn init_viewer_system(
    vehicle_query: Query<InitVehicleQueryData>,
    viewable_query: Query<(&view::Viewable,)>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    for vehicle in vehicle_query {
        messages.write_batch(vehicle.viewable.broadcast_new(|| {
            Some(proto::Update::from(proto::NewVehicle {
                id:       vehicle.viewable.id,
                ty:       vehicle.vehicle.ty.0,
                name:     vehicle.named.name.clone(),
                location: make_proto_location(vehicle.location, &viewable_query)?,
            }))
        }));
    }
}

#[derive(QueryData)]
struct InitVehicleQueryData {
    vehicle:  &'static Vehicle,
    location: &'static Location,
    named:    &'static view::Named,
    viewable: &'static view::Viewable,
}

fn incr_viewer_system(
    mut throttle: view::BroadcastThrottle,
    vehicle_query: Query<(&Location, &view::Viewable), With<Vehicle>>,
    viewable_query: Query<(&view::Viewable,)>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    if !throttle.should_run() {
        return;
    }

    for (location, viewable) in vehicle_query {
        messages.write_batch(viewable.broadcast_update(|_| {
            make_proto_location(location, &viewable_query).map(|location| {
                proto::Update::UpdateVehicleLocation(proto::UpdateVehicleLocation {
                    id: viewable.id,
                    location,
                })
            })
        }));
    }
}

fn make_proto_location(
    location: &Location,
    viewable_query: &Query<(&view::Viewable,)>,
) -> Option<proto::VehicleLocation> {
    match *location {
        Location::Building { building, interior_pos, speed } => {
            let (viewable,) = viewable_query.log_get(building)?;
            Some(proto::VehicleLocation::Building { building: viewable.id, interior_pos, speed })
        }
        Location::Rail { conduit, distance_from_alpha, speed_from_alpha } => {
            let (viewable,) = viewable_query.log_get(conduit)?;
            Some(proto::VehicleLocation::Rail {
                conduit: viewable.id,
                distance_from_alpha,
                speed_from_alpha,
            })
        }
    }
}

fn update_culling_rect_system(
    vehicle_query: Query<(&Location, &mut view::CullingRect)>,
    culling_rect_query: Query<&view::CullingRect, Without<Location>>,
) {
    for (location, mut culling_rect) in vehicle_query {
        let parent_entity = match *location {
            Location::Building { building, .. } => building,
            Location::Rail { conduit, .. } => conduit,
        };
        if let Some(&parent_rect) = culling_rect_query.log_get(parent_entity) {
            *culling_rect = parent_rect;
        }
    }
}
