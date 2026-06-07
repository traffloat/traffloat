use bevy::app::{self, App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::name::Name;
use bevy::ecs::query::With;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{EntityCommand, Query};
use bevy::ecs::world::{EntityWorldMut, World};
use bevy::math::Vec3;
use bevy::reflect::Reflect;
use traffloat_proto::proto;

use crate::util::{AllSystemSets, QueryExt};
use crate::{graph, view};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Resident>();
        app.register_type::<InteractionSlots>();
        app.register_type::<InteractingWith>();
        app.register_type::<InteractingResidents>();
        app.register_type::<NextResidentId>();

        app.init_resource::<NextResidentId>();
        app.init_resource::<Conf>();

        app.add_systems(app::Update, init_viewer_system.in_set(view::SendUpdatesSystemSet::Init));
        app.add_systems(
            app::Update,
            incr_viewer_system
                .after(AllSystemSets::<graph::ViewSystemSets>::default())
                .in_set(view::SendUpdatesSystemSet::Incr),
        );
    }
}

#[derive(Component, Reflect)]
pub struct Resident {
    pub name: String,
}

#[derive(Component, Reflect)]
pub enum Location {
    /// The resident is moving inside a building and not bound to a specific facility.
    Building {
        /// A `Building` entity.
        entity:       Entity,
        /// The position of the resident within the building, in local coordinates.
        /// `interior_pos.length()` should be approximately less than the building's radius.
        interior_pos: Vec3,
    },
    /// The resident is interacting with a facility.
    Facility {
        /// A `Facility` entity with [`InteractionSlots`].
        entity: Entity,
    },
    /// The resident is moving through a corridor.
    ///
    /// For practical reasons, we do not care about the cross-sectional position
    /// of the resident in the corridor, since that does not affect any behavior.
    Corridor {
        /// A `Corridor` entity.
        entity:              Entity,
        /// The distance of the resident from the alpha endpoint.
        /// `distance_from_alpha` should be approximately between 0 and the corridor's length.
        distance_from_alpha: f32,
    },
    // vehicle
}

/// Defines how residents can interact with a facility.
/// Component on facilities.
#[derive(Component, Reflect)]
pub struct InteractionSlots {
    pub slots: Vec<InteractionSlot>,
}

#[derive(Reflect)]
pub struct InteractionSlot {
    /// Maximum number of residents that can fit in this slot.
    pub capacity: u32,
    /// Current number of residents in this slot.
    pub usage:    u32,
}

/// The facility that a resident is currently interacting with.
///
/// Component on residents, only when they are interacting with a facility.
/// Removed when the interaction stops.
#[derive(Component, Reflect)]
#[relationship(relationship_target = InteractingResidents)]
pub struct InteractingWith {
    #[relationship]
    pub facility:   Entity,
    /// The index in [`InteractionSlots::slots`] that the resident is currently using.
    pub slot_index: usize,
}

/// List of residents currently interacting with a facility. Component on facilities.
#[derive(Component, Reflect)]
#[relationship_target(relationship = InteractingWith)]
pub struct InteractingResidents(Vec<Entity>);

#[derive(Resource, Reflect, Default)]
struct NextResidentId(u32);

impl NextResidentId {
    fn next(&mut self) -> u32 {
        self.0 = self.0.strict_add(1);
        self.0
    }
}

#[derive(Resource, Reflect)]
pub struct Conf {
    /// Distance per second within a building/corridor.
    pub standard_walking_speed: f32,
}

impl Default for Conf {
    fn default() -> Self { Self { standard_walking_speed: 1.0 } }
}

pub struct SpawnCommand {
    pub building: Entity,
}

impl EntityCommand for SpawnCommand {
    fn apply(self, mut entity: EntityWorldMut) {
        let id = entity.resource_mut::<NextResidentId>().next();

        entity.insert((
            Name::new(format!("Resident {id}")),
            Resident { name: id.to_string() },
            Location::Building { entity: self.building, interior_pos: Vec3::ZERO },
        ));

        entity.reborrow_scope(|entity| view::AddViewableCommand.apply(entity));
    }
}

fn init_viewer_system(
    resident_query: Query<(&Resident, &Location, &view::Viewable)>,
    viewable_query: Query<&view::Viewable>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    for (resident, location, viewable) in resident_query {
        messages.write_batch(viewable.broadcast_new(|| {
            Some(proto::Update::NewResident(proto::NewResident {
                id:       viewable.id,
                name:     resident.name.clone(),
                location: make_proto_location(location, &viewable_query)?,
            }))
        }));
    }
}

fn incr_viewer_system(
    mut throttle: view::BroadcastThrottle,
    resident_query: Query<(&Location, &view::Viewable), With<Resident>>,
    viewable_query: Query<&view::Viewable>,
    mut messages: MessageWriter<view::SentUpdate>,
) {
    if !throttle.should_run() {
        return;
    }

    for (location, viewable) in resident_query {
        messages.write_batch(viewable.broadcast_update(|level| {
            make_proto_location(location, &viewable_query).map(|location| {
                proto::Update::UpdateResidentLocation(proto::UpdateResidentLocation {
                    id: viewable.id,
                    location,
                })
            })
        }));
    }
}

fn make_proto_location(
    location: &Location,
    viewable_query: &Query<&view::Viewable>,
) -> Option<proto::ResidentLocation> {
    Some(match *location {
        Location::Building { entity, interior_pos } => proto::ResidentLocation::Building {
            building: viewable_query.log_get(entity)?.id,
            interior_pos,
            speed: Vec3::ZERO, // TODO
        },
        Location::Corridor { entity, distance_from_alpha } => proto::ResidentLocation::Corridor {
            corridor:   viewable_query.log_get(entity)?.id,
            linear_pos: distance_from_alpha,
            speed:      0.0, // TODO
        },
        Location::Facility { entity } => {
            proto::ResidentLocation::Facility { facility: viewable_query.log_get(entity)?.id }
        }
    })
}
