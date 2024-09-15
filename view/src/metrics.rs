//! A metric is a type of viewable attribute for an entity.

use std::any::type_name;
use std::time::Duration;
use std::{alloc, iter, mem};

use bevy::app::{self, App};
use bevy::ecs::component::{Component, ComponentDescriptor, ComponentId, StorageType};
use bevy::ecs::entity::Entity;
use bevy::ecs::event::{Event, EventWriter};
use bevy::ecs::query::{QueryData, QueryFilter};
use bevy::ecs::schedule::{IntoSystemConfigs, Schedules, SystemConfigs, SystemSet};
use bevy::ecs::system::{
    Commands, EntityCommand, Query, Res, StaticSystemParam, SystemBuilder, SystemParam,
};
use bevy::ecs::world::{Command, FilteredEntityMut, World};
use bevy::ptr::OwningPtr;
use bevy::time::{Time, Timer, TimerMode};
use rand::{thread_rng, Rng};
use rand_distr::StandardNormal;
use traffloat_base::partition::AppExt;

use crate::{viewable, viewer};

#[cfg(test)]
mod tests;

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) { app.add_partitioned_event::<UpdateMetricEvent>(); }
}

/// The identifier used to distinguish between types of metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Component)]
pub struct Type(pub Entity);

/// Creates a new type and initializes the corresponding systems.
pub fn create_type(commands: &mut Commands, def: TypeDef) -> Type {
    let entity = commands.spawn_empty();
    let ty = Type(entity.id());

    commands.push(CreateTypeCommand { ty, def });
    ty
}

/// Creates a new type of metric.
pub struct CreateTypeCommand {
    ty:  Type,
    def: TypeDef,
}

impl Command for CreateTypeCommand {
    fn apply(self, world: &mut World) {
        let value_comp_id = register_dynamic_component::<Value>(world, self.ty, StorageType::Table);
        let sub_comp_id =
            register_dynamic_component::<Subscription>(world, self.ty, StorageType::SparseSet);

        world.entity_mut(self.ty.0).insert((
            SubscriberComponentId(sub_comp_id),
            ValueComponentId(value_comp_id),
            self.def,
        ));

        let value_broadcast_system = make_value_broadcast_system(world, self.ty);
        world.resource_mut::<Schedules>().add_systems(app::Update, value_broadcast_system);
    }
}

#[derive(Component)]
struct SubscriberComponentId(ComponentId);

#[derive(Component)]
struct ValueComponentId(ComponentId);

/// Configuration for a metric type.
#[derive(Component)]
pub struct TypeDef {
    /// The period between broadcasts of the metric value to viewers.
    pub update_frequency: Duration,
}

/// A [`SystemParam`] to access the registered metric types.
#[derive(SystemParam)]
pub struct Types<'w, 's>(Query<'w, 's, (Entity, &'static TypeDef)>);

impl<'w, 's> Types<'w, 's> {
    /// Get a fluid type definition by type ID.
    #[must_use]
    pub fn get(&self, ty: Type) -> &TypeDef {
        self.0.get(ty.0).expect("reference to unknown metric type").1
    }

    /// Iterates over all known metric types.
    pub fn iter(&self) -> impl Iterator<Item = (Type, &TypeDef)> {
        self.0.iter().map(|(ty, def)| (Type(ty), def))
    }
}

/// The dynamic component type attached to entities storing the value of this metric.
pub struct Value {
    /// The actual magnitude of the metric value.
    pub magnitude: f32,
}

/// The dynamic component type attached to viewers to indicate that
/// the viewer should receive metrics of this type.
pub struct Subscription {
    /// The standard deviation of noise received by the user.
    ///
    /// The value sent to clients would be in the range
    /// `(value - noise)..=(value + noise)`.
    pub noise_sd: f32,
}

fn register_dynamic_component<T>(world: &mut World, ty: Type, storage: StorageType) -> ComponentId {
    /// # Safety
    /// `x` must points to a valid value of type `T`.
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
        let &SubscriberComponentId(subscriber_comp_id) = world
            .entity(self.ty.0)
            .get::<SubscriberComponentId>()
            .expect("metrics::Type refers to a non-metric or uninitialized entity");
        let mut viewer = world.entity_mut(self.viewer);

        // Safety:
        // 1. Config is loaded and populated by component IDs from the same world.
        // 2. ptr is a valid reference from a fresh OwningPtr.
        OwningPtr::make(self.subscription, |ptr| unsafe {
            viewer.insert_by_id(subscriber_comp_id, ptr)
        });
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
        let &SubscriberComponentId(subscriber_comp_id) = world
            .entity(self.ty.0)
            .get::<SubscriberComponentId>()
            .expect("metrics::Type refers to a non-metric or uninitialized entity");
        let mut viewer = world.entity_mut(self.viewer);

        viewer.remove_by_id(subscriber_comp_id);
    }
}

/// Notifies a viewer that a metric has been updated.
#[derive(Event)]
pub struct UpdateMetricEvent {
    /// The viewer to be notified.
    pub viewer:    viewer::Sid,
    /// The viewable that the metric is updated for.
    pub viewable:  viewable::Sid,
    /// The type of metric updated.
    pub ty:        Type,
    /// The updated metric magnitude, with noise included.
    pub magnitude: f32,
}

/// A system set to expose the value feeder system for a specific type.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct ValueFeederSystemSet(pub Type);

/// Creates a system that updates the magnitude of a metric type
/// for each entity matching `Query<OtherComps, Filter>`.
///
/// # Panics
/// Panics if the type is not initialized yet.
pub fn make_value_feeder_system<OtherComps, Filter, OtherSystemParams, FeederFn>(
    world: &mut World,
    feeder: FeederFn,
    ty: Type,
) -> SystemConfigs
where
    OtherComps: QueryData + 'static,
    Filter: QueryFilter + 'static,
    OtherSystemParams: SystemParam + 'static,
    FeederFn: Fn(&mut FilteredEntityMut<'_>, &OtherSystemParams::Item<'_, '_>) -> f32,
    FeederFn: Send + Sync + 'static,
{
    let &SubscriberComponentId(subscriber_comp_id) = world
        .entity(ty.0)
        .get::<SubscriberComponentId>()
        .expect("metrics::Type refers to a non-metric or uninitialized entity");
    let &ValueComponentId(value_comp_id) = world
        .entity(ty.0)
        .get::<ValueComponentId>()
        .expect("metrics::Type refers to a non-metric or uninitialized entity");

    SystemBuilder::<(Commands,)>::new(world)
        .builder::<Query<()>>(|builder| {
            builder.with_id(subscriber_comp_id);
        })
        .builder::<Query<FilteredEntityMut, Filter>>(|builder| {
            builder.optional(|builder| {
                builder.mut_id(value_comp_id);
            });
            builder.data::<OtherComps>();
        })
        .param::<StaticSystemParam<OtherSystemParams>>()
        .build(
            move |mut commands: Commands,
                  check_has_viewer_query: Query<()>,
                  mut query: Query<FilteredEntityMut, Filter>,
                  other_params: StaticSystemParam<OtherSystemParams>| {
                if check_has_viewer_query.is_empty() {
                    return;
                }

                let other_params = other_params.into_inner();

                query.iter_mut().for_each(|mut entity| {
                    let magnitude = feeder(&mut entity, &other_params);

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
                });
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
        // Safety: ptr is used only within `OwningPtr::make` closure.
        OwningPtr::make(Value { magnitude: self.magnitude }, |ptr| unsafe {
            world.entity_mut(entity).insert_by_id(self.comp_id, ptr);
        });
    }
}

fn make_value_broadcast_system(world: &mut World, ty: Type) -> SystemConfigs {
    let &SubscriberComponentId(subscriber_comp_id) = world
        .entity(ty.0)
        .get::<SubscriberComponentId>()
        .expect("metrics::Type refers to a non-metric or uninitialized entity");
    let &ValueComponentId(value_comp_id) = world
        .entity(ty.0)
        .get::<ValueComponentId>()
        .expect("metrics::Type refers to a non-metric or uninitialized entity");
    let def = world
        .entity(ty.0)
        .get::<TypeDef>()
        .expect("metrics::Type refers to a non-metric or uninitialized entity");

    let mut timer = Timer::new(def.update_frequency, TimerMode::Repeating);

    SystemBuilder::<(Res<Time>, EventWriter<UpdateMetricEvent>)>::new(world)
        .builder::<Query<FilteredEntityMut>>(|builder| {
            builder.ref_id(subscriber_comp_id);
            builder.data::<&viewer::Sid>();
        })
        .builder::<Query<FilteredEntityMut>>(|builder| {
            builder.ref_id(value_comp_id);
            builder.data::<&viewable::Viewers>();
            builder.data::<&viewable::Sid>();
        })
        .build(
            move |time: Res<Time>,
                  mut events: EventWriter<UpdateMetricEvent>,
                  viewers_query: Query<FilteredEntityMut>,
                  viewables_query: Query<FilteredEntityMut>| {
                timer.tick(time.delta());
                if !timer.finished() {
                    return;
                }

                let viewers_query = &viewers_query;

                let event_ctors = viewables_query.iter().flat_map(move |viewable_fem| {
                    let value_ptr =
                        viewable_fem.get_by_id(value_comp_id).expect("requested in query");
                    // Safety: Value components must have type Value
                    let &Value { magnitude } = unsafe { value_ptr.deref::<Value>() };

                    let viewable_viewers =
                        viewable_fem.get::<viewable::Viewers>().expect("requested in query");
                    viewable_viewers.iter().filter_map(move |viewer_entity| {
                        let viewer_fem = viewers_query.get(viewer_entity).ok()?;
                        let sub_ptr =
                            viewer_fem.get_by_id(subscriber_comp_id).expect("requested in query");
                        let viewer_sid =
                            *viewer_fem.get::<viewer::Sid>().expect("requested in query");
                        let viewable_sid =
                            *viewable_fem.get::<viewable::Sid>().expect("requested in query");
                        // Safety: subscription component must have type Subscription
                        let &Subscription { noise_sd } = unsafe { sub_ptr.deref::<Subscription>() };
                        Some(move |z: f32| UpdateMetricEvent {
                            viewer: viewer_sid,
                            viewable: viewable_sid,
                            ty,
                            magnitude: magnitude + z * noise_sd,
                        })
                    })
                });

                events.send_batch(
                    iter::zip(event_ctors, thread_rng().sample_iter(StandardNormal))
                        .map(|(event_ctor, z)| event_ctor(z)),
                );
            },
        )
        .after(ValueFeederSystemSet(ty))
}
