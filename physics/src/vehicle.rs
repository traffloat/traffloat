use std::collections::VecDeque;
use std::time::Duration;

use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::{EntityEvent, };
use bevy::ecs::message::MessageWriter;
use bevy::ecs::name::Name;
use bevy::ecs::query::{QueryData, With, Without};
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::{IntoScheduleConfigs, SystemSet};
use bevy::ecs::system::{Commands, EntityCommand, Query, SystemParam};
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
pub mod rail;
pub use rail::Rail;
use traffloat_proto::proto::{self, AlphaOrBeta};

use crate::graph::{Corridor, conduit};
use crate::util::{
    self, Alpha, EntityWorldMutExt, QueryExt, Which, WorldExt, run_stateless_closure,
};
use crate::{CleanupAppExt, fluid, view};

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
        app.add_systems(
            app::Update,
            update_culling_rect_system
                .in_set(view::SendUpdatesSystemSet::Cull)
                .in_set(UpdateCullingRectSystemSet),
        );

        app.add_cleanup_hook(Types::cleanup_hook);

        util::configure_enum_system_set::<SystemSets>(app, app::FixedUpdate);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet, strum::EnumIter)]
pub enum SystemSets {
    Pathfinding,
    Motion,
    Propulsion,
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
#[require(motion::Intent, propulsion::Status)]
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

impl ListOnRail {
    pub fn partition_point_by_location(
        &self,
        dist: f32,
        mut loc_fn: impl FnMut(Entity) -> Option<Location>,
    ) -> Option<usize> {
        self.partition_point(dist, |e| match loc_fn(e)? {
            Location::Building { .. } => None,
            Location::Rail { distance_from_alpha, .. } => Some(distance_from_alpha),
        })
    }

    pub fn partition_point(
        &self,
        dist: f32,
        mut dist_fn: impl FnMut(Entity) -> Option<f32>,
    ) -> Option<usize> {
        let mut valid = true;
        let pp = self.deque.partition_point(|&e| match dist_fn(e) {
            Some(d) => d < dist,
            None => {
                valid = false;
                false
            }
        });
        valid.then_some(pp)
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<Entity> { self.deque.get(index).copied() }

    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ { self.deque.iter().copied() }
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
    /// Reaction time that drivers reserve for safety
    /// in addition to the braking distance and safety headroom.
    pub reaction_time:           Duration,
    /// Distance that drivers aim to stay away from the vehicle ahead for safety.
    ///
    /// When set to 0, stationary vehicles will aim to stick together head-to-tail.
    pub safety_headroom:         f32,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            standard_drifting_speed: 3.0,
            reaction_time:           Duration::from_millis(800),
            safety_headroom:         1.0,
        }
    }
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
            .enumerate()
            .map(|(index, cpmt)| {
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

                let cpmt_name = format!("Vehicle {name} compartment {index}");
                move |cpmt_entity: &mut EntityWorldMut| {
                    let cmpt_entity_id = cpmt_entity.id();
                    add_edge_cmd.alpha = cmpt_entity_id;

                    cpmt_entity.insert(Name::new(cpmt_name));
                    cpmt_entity.reborrow_scope(|child| add_storage_cmd.apply(child));
                    cpmt_entity.world_scope(|world| {
                        let vent_entity = world.spawn(CompartmentVentOf(cmpt_entity_id));
                        add_edge_cmd.apply(vent_entity);
                    });
                }
            })
            .collect();

        entity.insert((
            Name::new(format!("Vehicle {name}")),
            view::Named { name: name.clone() },
            Vehicle { ty: self.ty, mass },
        ));
        entity.with_related_entities::<CompartmentOf>(|spawner| {
            for post_spawn in cpmts {
                let mut child = spawner.spawn_empty();
                post_spawn(&mut child);
            }
        });

        entity.reborrow_scope(|entity| view::AddViewableCommand.apply(entity));
        AttemptLocationTransitionCommand { new_location: self.location, entry_method: () }
            .apply(entity);
    }
}

pub struct AttemptLocationTransitionCommand<Ab> {
    pub new_location: Location,
    /// Whether the entering edge is an alpha or beta edge.
    /// This is currently unused for entering buildings, but should still be set correctly.
    pub entry_method: Ab,
}

impl<Ab: EntryMethod> EntityCommand for AttemptLocationTransitionCommand<Ab> {
    type Out = ();

    fn apply(self, mut entity: EntityWorldMut) {
        let entity_id = entity.id();

        if let Some(entry) = self.entry_method.into_proto() {
            // only check if this is a regular entry
            match self.new_location {
                Location::Building { .. } => {
                    // TODO check building capacity
                }
                Location::Rail { conduit, .. } => {
                    let result = check_rail_entry(entity.world(), entity_id, conduit, entry);
                    if result.is_err() {
                        return;
                    }
                }
            }
        }

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
                    entity.world_scope(|world| exit_from_rail(world, conduit, entity_id));
                }
            }
        }

        entity.insert(self.new_location);
        if let Some(mut desired) = entity.log_get_mut::<propulsion::Desired>() {
            // reset desired propulsion to a universally acceptable value,
            // then let the motion control system overwrite this.
            *desired = propulsion::Desired::Stationary;
        }

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
            Location::Rail { conduit, distance_from_alpha, .. } => {
                run_stateless_closure(world, move |params: EnterRailParams<'_, '_>| {
                    enter_rail(params, conduit, entity_id, distance_from_alpha, self.entry_method);
                });
            }
        }

        world.entity_mut(entity_id).trigger(move |entity| LocationTransitionEvent {
            entity,
            entry_method: self.entry_method.into_proto(),
        });
    }
}

/// An entity event triggered on successful location transition.
#[derive(Debug, Clone, Copy, EntityEvent)]
pub struct LocationTransitionEvent {
    #[entity_event]
    pub entity:       Entity,
    pub entry_method: Option<AlphaOrBeta>,
}

pub trait EntryMethod: Copy + Send + Sync + 'static {
    fn into_proto(self) -> Option<AlphaOrBeta>;

    fn into_which(self) -> Option<impl Which>;
}

impl<Ab: Which> EntryMethod for Ab {
    fn into_proto(self) -> Option<AlphaOrBeta> { Some(self.proto()) }

    fn into_which(self) -> Option<impl Which> { Some(self) }
}

impl EntryMethod for () {
    fn into_proto(self) -> Option<AlphaOrBeta> { None }

    fn into_which(self) -> Option<impl Which> { None::<Alpha> }
}

enum RailEntryCheck {
    InvalidEcs,
    WrongDirection,
    ReservedByExternal,
    PhysicallyBlocked,
}

fn check_rail_entry(
    world: &World,
    vehicle: Entity,
    conduit: Entity,
    entry: AlphaOrBeta,
) -> Result<(), RailEntryCheck> {
    let Some(reserved) = world.log_get::<rail::Reservation>(conduit) else {
        return Err(RailEntryCheck::InvalidEcs);
    };
    let Some(reserved) = reserved.inner else { return Ok(()) };
    if reserved.direction != rail::ReservedDirection::from_entry(entry) {
        tracing::warn!(
            "Vehicle {vehicle:?} attempts to enter conduit {conduit:?} exit point {entry:?}"
        );
        return Err(RailEntryCheck::WrongDirection);
    }

    if reserved.external_vehicle.is_some() {
        return Err(RailEntryCheck::ReservedByExternal);
    }

    let Some(vehicles) = world.log_get::<ListOnRail>(conduit) else {
        return Err(RailEntryCheck::InvalidEcs);
    };
    let last_vehicle = match entry {
        AlphaOrBeta::Alpha => vehicles.deque.front().copied(),
        AlphaOrBeta::Beta => vehicles.deque.back().copied(),
    };
    if let Some(last_vehicle) = last_vehicle {
        let Some(location) = world.log_get::<Location>(last_vehicle) else {
            return Err(RailEntryCheck::InvalidEcs);
        };
        let &Location::Rail { distance_from_alpha, .. } = location else {
            tracing::error!("Vehicle in list must be on rail");
            return Err(RailEntryCheck::InvalidEcs);
        };

        let distance_from_entry = match entry {
            AlphaOrBeta::Alpha => distance_from_alpha,
            AlphaOrBeta::Beta => {
                let Some(of_corridor) = world.log_get::<conduit::OfCorridor>(conduit) else {
                    return Err(RailEntryCheck::InvalidEcs);
                };
                let Some(corridor) = world.log_get::<Corridor>(of_corridor.0) else {
                    return Err(RailEntryCheck::InvalidEcs);
                };
                corridor.length - distance_from_alpha
            }
        };

        let Some(vehicle_data) = world.log_get::<Vehicle>(vehicle) else {
            return Err(RailEntryCheck::InvalidEcs);
        };
        let vehicle_def = world.resource::<Types>().get(vehicle_data.ty);
        let vehicle_length = vehicle_def.physical.length;

        if distance_from_entry < vehicle_length * 0.5 {
            return Err(RailEntryCheck::PhysicallyBlocked);
        }
    }

    Ok(())
}

fn exit_from_rail(world: &mut World, conduit: Entity, vehicle_entity: Entity) {
    let mut clear_reservation = false;

    if let Some(mut list) = world.log_get_mut::<ListOnRail>(conduit) {
        if list.deque.back() == Some(&vehicle_entity) {
            // retain will scan from front,
            // but back is much more likely than list.deque[1]
            list.deque.pop_back();
        } else {
            list.deque.retain(|&e| e != vehicle_entity);
        }

        if list.deque.is_empty() {
            clear_reservation = true;
        }
    }

    if clear_reservation
        && let Some(mut reservation) = world.log_get_mut::<rail::Reservation>(conduit)
        && let Some(rail::ReservationInner { external_vehicle: None, .. }) = reservation.inner
    {
        // Last vehicle exits, direction is unreserved now.
        reservation.inner = None;
    }
}

#[derive(SystemParam)]
struct EnterRailParams<'w, 's> {
    conduit_query: Query<'w, 's, (Option<&'static mut ListOnRail>, &'static mut rail::Reservation)>,
    location_query: Query<'w, 's, &'static Location>,
    commands:       Commands<'w, 's>,
}

fn enter_rail<Ab: EntryMethod>(
    mut params: EnterRailParams,
    conduit: Entity,
    vehicle_entity: Entity,
    distance_from_alpha: f32,
    entry_method: Ab,
) {
    let Some((list, mut reservation)) = params.conduit_query.log_get_mut(conduit) else { return };
    match list {
        Some(mut list) => match entry_method.into_proto() {
            Some(AlphaOrBeta::Alpha) => list.deque.push_front(vehicle_entity),
            Some(AlphaOrBeta::Beta) => list.deque.push_back(vehicle_entity),
            None => {
                let pos = list.partition_point_by_location(distance_from_alpha, |e| {
                    params.location_query.log_get(e).copied()
                });
                list.deque.insert(pos.unwrap_or(0), vehicle_entity);
            }
        },
        None => {
            params.commands.entity(conduit).insert(ListOnRail { deque: [vehicle_entity].into() });
        }
    }

    match reservation.inner {
        None => {
            let Some(entry_endpoint) = entry_method.into_proto() else {
                // When a valid savefile is loaded with a vehicle on it,
                // the rail must have been reserved by this vehicle.
                // This branch implies that the savefile is invalid.
                tracing::error!(
                    "Random rail entry is only allowed on reserved rails during savefile load"
                );
                return;
            };
            reservation.inner = Some(rail::ReservationInner {
                direction:        rail::ReservedDirection::from_entry(entry_endpoint),
                external_vehicle: None,
            });
        }
        Some(ref mut inner) => {
            if let Some(entry_endpoint) = entry_method.into_proto() {
                let entry_direction = rail::ReservedDirection::from_entry(entry_endpoint);
                if inner.direction != entry_direction {
                    tracing::error!("Vehicle entered rail from the wrong direction");
                    return;
                }
            }

            if inner.external_vehicle == Some(vehicle_entity) {
                // the vehicle is no longer external now
                inner.external_vehicle = None;
            }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct UpdateCullingRectSystemSet;

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
