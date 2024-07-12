//! A metric is a type of viewable attribute for an entity.

use std::any::type_name;
use std::{alloc, mem};

use bevy::app::{self, App};
use bevy::ecs::component::{ComponentDescriptor, ComponentId, StorageType};
use bevy::ecs::event::{Event, EventWriter};
use bevy::ecs::query::{QueryData, QueryFilter};
use bevy::ecs::schedule::{IntoSystemConfigs, Schedules, SystemConfigs, SystemSet};
use bevy::ecs::system::{EntityCommand, StaticSystemParam, SystemParam};
use bevy::ecs::world::{Command, FilteredEntityMut};
use bevy::prelude::{Commands, Entity, Query, Resource, SystemBuilder, World};
use bevy::ptr::OwningPtr;
use rand::{thread_rng, Rng};
use rand_distr::StandardNormal;

use crate::viewable;

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShowEvent>();
        app.add_event::<HideEvent>();
        app.add_event::<UpdateMetricEvent>();
    }
}

#[derive(Resource)]
pub struct Config {
    types: Vec<ConfigTypeEntry>,
}

impl Config {
    pub fn get_type(&self, ty: Type) -> &TypeDef {
        let entry = self
            .types
            .get(ty.0)
            .expect("a Type was constructed without a matching entry in the Config");
        &entry.def
    }
}

/// The identifier used to distinguish between types of metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Type(pub usize);

struct ConfigTypeEntry {
    sub_comp_id:   ComponentId,
    value_comp_id: ComponentId,
    def:           TypeDef,
}

pub struct TypeDef {}

/// The dynamic component type attached to entities storing the value of this metric.
pub struct Value {
    pub magnitude: f32,
}

/// The dynamic marker component type attached to viewers to indicate that
/// the viewer should receive metrics of this type.
pub struct Subscription {
    /// The standard deviation of noise received by the user.
    ///
    /// The value sent to clients would be in the range
    /// `(value - noise)..=(value + noise)`.
    pub noise_sd: f32,
}

pub struct CreateTypeCommand {
    def: TypeDef,
}

fn register_dynamic_component<T>(world: &mut World, ty: Type, storage: StorageType) -> ComponentId {
    // # Safety
    // `x` must points to a valid value of type `T`.
    unsafe fn drop_ptr<T>(x: OwningPtr<'_>) { unsafe { x.drop_as::<T>() } }
    let descriptor = unsafe {
        ComponentDescriptor::new_with_layout(
            format!("{} #{}", type_name::<T>(), ty.0),
            storage,
            alloc::Layout::new::<T>(),
            mem::needs_drop::<T>().then_some(drop_ptr::<T>),
        )
    };

    world.init_component_with_descriptor(descriptor)
}

impl Command for CreateTypeCommand {
    fn apply(self, world: &mut World) {
        let ty = Type(world.resource::<Config>().types.len());

        let value_comp_id = register_dynamic_component::<Value>(world, ty, StorageType::Table);
        let sub_comp_id =
            register_dynamic_component::<Subscription>(world, ty, StorageType::SparseSet);

        world.resource_mut::<Config>().types.push(ConfigTypeEntry {
            sub_comp_id,
            value_comp_id,
            def: self.def,
        });

        let value_broadcast_system = make_value_broadcast_system(world, ty);
        world.resource_mut::<Schedules>().add_systems(app::Update, value_broadcast_system);
    }
}

/// Adds the viewer as a subscriber of the metric type.
pub struct SubscribeCommand {
    /// The viewer entity.
    pub viewer:       Entity,
    /// The type of metric to subscribe to.
    pub ty:           Type,
    /// The subscription object to create.
    pub subscription: Subscription,
}

impl Command for SubscribeCommand {
    fn apply(self, world: &mut World) {
        let types = &world.resource::<Config>().types;
        let &ConfigTypeEntry { sub_comp_id, .. } =
            types.get(self.ty.0).expect("unknown metric type");

        let mut viewer = world.entity_mut(self.viewer);

        // Safety:
        // 1. Config is loaded and populated by component IDs from the same world.
        // 2. ptr is a valid reference from a fresh OwningPtr.
        OwningPtr::make(self.subscription, |ptr| unsafe { viewer.insert_by_id(sub_comp_id, ptr) });
    }
}

/// Removes the viewer as a subscriber of the metric type.
pub struct UnsubscribeCommand {
    /// The viewer entity.
    pub viewer: Entity,
    /// The type of metric to subscribe to.
    pub ty:     Type,
}

impl Command for UnsubscribeCommand {
    fn apply(self, world: &mut World) {
        let types = &world.resource::<Config>().types;
        let &ConfigTypeEntry { sub_comp_id, .. } =
            types.get(self.ty.0).expect("unknown metric type");

        let mut viewer = world.entity_mut(self.viewer);

        viewer.remove_by_id(sub_comp_id);
    }
}

/// The client should start displaying an entity.
#[derive(Event)]
pub struct ShowEvent {
    pub viewer:   Entity,
    pub viewable: Entity,
}

/// The client should stop displaying an entity.
#[derive(Event)]
pub struct HideEvent {
    pub viewer:   Entity,
    pub viewable: Entity,
}

/// The client should gradually change the metric display to the set value.
#[derive(Event)]
pub struct UpdateMetricEvent {
    pub viewer:    Entity,
    pub viewable:  Entity,
    pub magnitude: f32,
}

/// A system set to expose the value feeder system for a specific type.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct ValueFeederSystemSet(pub Type);

/// Creates a system that updates the magnitude of a metric type
/// for each entity matching `Query<OtherComps, Filter>`.
pub fn make_value_feeder_system<Filter, OtherComps, OtherSystemParams, FeederFn>(
    world: &mut World,
    feeder: FeederFn,
    ty: Type,
) -> SystemConfigs
where
    OtherComps: QueryData + 'static,
    Filter: QueryFilter + 'static,
    OtherSystemParams: SystemParam + 'static,
    FeederFn: Fn(OtherComps::Item<'_>, &OtherSystemParams::Item<'_, '_>) -> f32,
    FeederFn: Send + Sync + 'static,
{
    let &ConfigTypeEntry { sub_comp_id, value_comp_id, .. } = world
        .resource::<Config>()
        .types
        .get(ty.0)
        .expect("cannot build value feeder for uninitialized type");

    SystemBuilder::<(Commands,)>::new(world)
        .builder::<Query<()>>(|builder| {
            builder.with_id(sub_comp_id);
        })
        .builder::<Query<(FilteredEntityMut, OtherComps), Filter>>(|builder| {
            builder.optional(|builder| {
                builder.mut_id(value_comp_id);
            });
            builder.data::<OtherComps>();
        })
        .param::<StaticSystemParam<OtherSystemParams>>()
        .build(
            move |mut commands: Commands,
                  check_has_viewer_query: Query<()>,
                  mut query: Query<(FilteredEntityMut, OtherComps), Filter>,
                  other_params: StaticSystemParam<OtherSystemParams>| {
                if check_has_viewer_query.is_empty() {
                    return;
                }

                let other_params = other_params.into_inner();

                for (mut entity, other_comps) in query.iter_mut() {
                    let magnitude = feeder(other_comps, &other_params);

                    match entity.get_mut_by_id(value_comp_id) {
                        Some(value) => {
                            // Safety: Value components must have type Value
                            let mut value = unsafe { value.with_type::<Value>() };
                            value.magnitude = magnitude;
                        }
                        None => {
                            commands
                                .entity(entity.id())
                                .add(InitValueCommand { comp_id: value_comp_id, magnitude });
                        }
                    }
                }
            },
        )
        .in_set(ValueFeederSystemSet(ty))
}

struct InitValueCommand {
    comp_id:   ComponentId,
    magnitude: f32,
}

impl EntityCommand for InitValueCommand {
    fn apply(self, entity: Entity, world: &mut World) {
        OwningPtr::make(Value { magnitude: self.magnitude }, |ptr| unsafe {
            world.entity_mut(entity).insert_by_id(self.comp_id, ptr);
        })
    }
}

fn make_value_broadcast_system(world: &mut World, ty: Type) -> SystemConfigs {
    let &ConfigTypeEntry { sub_comp_id, value_comp_id, .. } = world
        .resource::<Config>()
        .types
        .get(ty.0)
        .expect("cannot build value feeder for uninitialized type");

    SystemBuilder::<(EventWriter<UpdateMetricEvent>,)>::new(world)
        .builder::<Query<FilteredEntityMut>>(|builder| {
            builder.ref_id(sub_comp_id);
        })
        .builder::<Query<(&viewable::Viewers, FilteredEntityMut)>>(|builder| {
            builder.ref_id(value_comp_id);
        })
        .build(
            move |mut events: EventWriter<UpdateMetricEvent>,
                  viewers_query: Query<FilteredEntityMut>,
                  viewables_query: Query<(&viewable::Viewers, FilteredEntityMut)>| {
                let viewers_query = &viewers_query;

                events.send_batch(viewables_query.iter().flat_map(
                    move |(viewable_viewers, viewable_entity)| {
                        let value_ptr =
                            viewable_entity.get_by_id(value_comp_id).expect("requested in query");
                        // Safety: Value components must have type Value
                        let &Value { magnitude } = unsafe { value_ptr.deref::<Value>() };
                        let viewable_id = viewable_entity.id();

                        viewable_viewers.viewers.iter().filter_map(move |&viewer_entity| {
                            let sub_ptr = viewers_query
                                .get(viewer_entity)
                                .ok()?
                                .get_by_id(sub_comp_id)
                                .expect("requested in query");
                            // Safety: subscription component must have type Subscription
                            let &Subscription { noise_sd } =
                                unsafe { sub_ptr.deref::<Subscription>() };
                            let z: f32 = thread_rng().sample(StandardNormal);
                            Some(UpdateMetricEvent {
                                viewer:    viewer_entity,
                                viewable:  viewable_id,
                                magnitude: magnitude + z * noise_sd,
                            })
                        })
                    },
                ));
            },
        )
        .after(ValueFeederSystemSet(ty))
}
