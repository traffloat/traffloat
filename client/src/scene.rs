//! A local mirror of the actual world based on incremental [proto](traffloat_proto) updates.

use std::collections::HashMap;

use bevy::app::{self, App, Plugin};
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::{Message, MessageReader};
use bevy::ecs::query::With;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::{
    ApplyDeferred, IntoScheduleConfigs, Schedulable, ScheduleConfigs, SystemSet,
};
use bevy::ecs::system::{Commands, ParamSet, Query, ResMut, ScheduleSystem, Single, SystemParam};
use bevy::ecs::world::{EntityWorldMut, World};
use bevy::math::Vec3;
use bevy::mesh::Mesh2d;
use bevy::picking::PickingSettings;
use bevy::reflect::Reflect;
use bevy::sprite_render::{ColorMaterial, MeshMaterial2d};
use bevy::transform::components::Transform;
use bevy_mod_config::{AppExt, Config, ReadConfig};
use enum_map::EnumMap;
use itertools::Itertools;
use strum::IntoEnumIterator;
use traffloat_physics::util::{EntityWorldMutExt, QueryExt, WorldExt};
use traffloat_physics::{try_log, view};
use traffloat_proto::proto;

use crate::ConfigManager;

pub mod building;
pub mod conduit;
pub mod corridor;
pub mod facility;

mod picking;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<ProtoId>();
        app.register_type::<IdRegistry>();

        app.init_resource::<IdRegistry>();
        app.init_resource::<FluidTypes>();
        app.init_config::<ConfigManager, Conf>("scene");
        app.add_plugins(picking::Plug);
        app.add_plugins(building::Plug);
        app.add_plugins(corridor::Plug);
        app.add_plugins(facility::Plug);
        app.add_plugins(conduit::Plug);
        app.add_systems(app::Update, react_config_system);

        for (prev, next) in HandlerClass::iter().tuple_windows() {
            app.configure_sets(app::Update, prev.before(next).in_set(AllHandlersSystemSet));
            app.add_systems(app::Update, ApplyDeferred.before(next).after(prev));
        }
        for class in HandlerClass::iter() {
            app.add_systems(app::Update, make_handle_update_system(class));
        }
    }
}

#[derive(Resource, Default, Reflect)]
pub struct IdRegistry {
    map: HashMap<proto::Id, TrackedId>,
}

impl IdRegistry {
    pub fn get_building(&self, id: proto::Id) -> Option<Entity> {
        match self.map.get(&id) {
            Some(TrackedId::Building(entity)) => Some(*entity),
            Some(v) => {
                tracing::error!(
                    "Expected received ID {id:?} to be a building, found {:?}",
                    <&'static str>::from(v),
                );
                None
            }
            None => {
                tracing::error!("Received unknown building id {id:?}");
                None
            }
        }
    }

    pub fn get_corridor(&self, id: proto::Id) -> Option<Entity> {
        match self.map.get(&id) {
            Some(TrackedId::Corridor(entity)) => Some(*entity),
            Some(v) => {
                tracing::error!(
                    "Expected received ID {id:?} to be a corridor, found {:?}",
                    <&'static str>::from(v),
                );
                None
            }
            None => {
                tracing::error!("Received unknown corridor id {id:?}");
                None
            }
        }
    }

    pub fn get_facility(&self, id: proto::Id) -> Option<Entity> {
        match self.map.get(&id) {
            Some(TrackedId::Facility(entity)) => Some(*entity),
            Some(v) => {
                tracing::error!(
                    "Expected received ID {id:?} to be a facility, found {:?}",
                    <&'static str>::from(v),
                );
                None
            }
            None => {
                tracing::error!("Received unknown facility id {id:?}");
                None
            }
        }
    }

    pub fn get_conduit(&self, id: proto::Id) -> Option<Entity> {
        match self.map.get(&id) {
            Some(TrackedId::Conduit(entity)) => Some(*entity),
            Some(v) => {
                tracing::error!(
                    "Expected received ID {id:?} to be a conduit, found {:?}",
                    <&'static str>::from(v),
                );
                None
            }
            None => {
                tracing::error!("Received unknown conduit id {id:?}");
                None
            }
        }
    }
}

#[derive(Reflect, strum::IntoStaticStr)]
enum TrackedId {
    Building(Entity),
    Corridor(Entity),
    Facility(Entity),
    Conduit(Entity),
}

#[derive(Component, Reflect)]
pub struct ProtoId(pub proto::Id);

/// Marks the viewer entity for singleplayer client.
#[derive(Component)]
struct SinglePlayerViewer;

pub fn setup_singleplayer(world: &mut World) {
    world.spawn((traffloat_physics::WorldObject, SinglePlayerViewer, view::Viewer::default()));
}

fn react_config_system(
    conf: ReadConfig<Conf>,
    viewer: Option<Single<&mut view::Viewer, With<SinglePlayerViewer>>>,
) {
    let Some(mut viewer) = viewer else { return };
    let conf = conf.read();
    let viewer = &mut **viewer;
    viewer.set_level(conf.subscription_level);
}

/// Rendering order, from back to front.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::EnumCount)]
pub enum Zorder {
    Building,
    BuildingWall,
    Corridor,
    CorridorWall,
    FacilityTaint,
    Facility,
    Conduit,
}

impl Zorder {
    #[expect(clippy::cast_precision_loss, reason = "COUNT < 2^(f32::MANTISSA_DIGITS)")]
    pub const fn z(self) -> f32 {
        0.25 + (self as u8 as f32) / (<Self as strum::EnumCount>::COUNT as f32) * 0.5
    }
}

#[derive(SystemParam)]
struct HandleUpdateParams<'w, 's> {
    updates: MessageReader<'w, 's, view::SentUpdate>,
    mux:     UpdateHandlerMux<'w, 's>,
    viewer:  Option<Single<'w, 's, Entity, With<SinglePlayerViewer>>>,
}

fn make_handle_update_system(class: HandlerClass) -> ScheduleConfigs<ScheduleSystem> {
    (move |params: HandleUpdateParams| handle_update_system(class, params)).in_set(class)
}

fn handle_update_system(class: HandlerClass, mut params: HandleUpdateParams) {
    let Some(viewer) = params.viewer else { return };

    for update in params.updates.read() {
        if update.viewers.contains(&*viewer) && UpdateHandlerMux::classify(&update.body) == class {
            tracing::info_span!("handle_update", update = ?<&'static str>::from(&update.body))
                .in_scope(|| {
                    tracing::debug!("Handle update {:?}", update.body);
                    params.mux.handle(&update.body);
                });
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, strum::EnumIter, SystemSet)]
enum HandlerClass {
    Meta,
    Spawn,
    MixedSpawn,
    Update,
    Despawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, SystemSet)]
pub struct AllHandlersSystemSet;

macro_rules! define_params {
    (
        $w:lifetime, $s:lifetime;
        $(
            $short:ident($param:ty),
        )*
    ) => {
        traffloat_macro_util::triangle! {
            define_params (p1);
            @expanded $w, $s;
            $($short($param),)*
        }
    };
    (
        @expanded $w:lifetime, $s:lifetime;
        $(
            ($($p1:ident)*) $short:ident($param:ty),
        )*
    ) => {
        #[derive(SystemParam)]
        struct UpdateHandlerMux<$w, $s>(recurse_nested_tuple!($w, $s; $($param,)*));

        impl<'w, 's> UpdateHandler for UpdateHandlerMux<'w, 's> {
            type Update = proto::Update;

            fn classify(update: &proto::Update) -> HandlerClass {
                match update {
                    $(
                        proto::Update::$short(update) => {
                            <$param as UpdateHandler>::classify(update)
                        }
                    )*
                }
            }

            fn handle(&mut self, update: &proto::Update) {
                match update {
                    $(
                        proto::Update::$short(update) => {
                            UpdateHandler::handle(&mut self.0 $(.$p1())* .p0(), update)
                        }
                    )*
                }
            }
        }
    }
}

macro_rules! recurse_nested_tuple {
    ($w:lifetime, $s:lifetime; ) => { () };
    ($w:lifetime, $s:lifetime; $param:ty, $($rest:ty,)*) => {
        ParamSet<$w, $s, ($param, recurse_nested_tuple!($w, $s; $($rest,)*))>
    };
}

trait UpdateHandler {
    type Update;

    fn classify(update: &Self::Update) -> HandlerClass;

    fn handle(&mut self, update: &Self::Update);
}

define_params! {
    'w, 's;
    NewBuilding(building::NewBuildingParams<'w, 's>),
    UpdateBuilding(building::UpdateBuildingParams<'w, 's>),
    UpdateBuildingFull(building::UpdateBuildingFullParams<'w, 's>),
    NewCorridor(corridor::NewCorridorParams<'w, 's>),
    UpdateCorridor(corridor::UpdateCorridorParams<'w, 's>),
    UpdateCorridorFull(corridor::UpdateCorridorFullParams<'w, 's>),
    SetCorridorEndpoint(corridor::SetCorridorEndpointParams<'w, 's>),
    NewFacility(facility::NewFacilityParams<'w, 's>),
    SetFacilityTaint(facility::SetFacilityTaintParams<'w, 's>),
    SetFacilityFluid(facility::SetFacilityFluidParams<'w, 's>),
    NewConduit(conduit::NewConduitParams<'w, 's>),
    UpdateFluidConduit(conduit::UpdateFluidConduitParams<'w, 's>),
    UpdateFluidConduitFull(conduit::UpdateFluidConduitFullParams<'w, 's>),
    RemoveViewable(RemoveViewableParams<'w, 's>),
    SetFluidTypes(SetFluidTypesParams<'w>),
}

#[derive(SystemParam)]
struct RemoveViewableParams<'w, 's> {
    commands: Commands<'w, 's>,
    ids:      ResMut<'w, IdRegistry>,
}

impl UpdateHandler for RemoveViewableParams<'_, '_> {
    type Update = proto::RemoveViewable;

    fn classify(_update: &Self::Update) -> HandlerClass { HandlerClass::Despawn }

    fn handle(&mut self, fixture: &proto::RemoveViewable) {
        match self.ids.map.remove(&fixture.id) {
            Some(TrackedId::Building(entity) | TrackedId::Corridor(entity)) => {
                self.commands.entity(entity).despawn();
            }
            Some(TrackedId::Facility(entity)) => {
                self.commands.entity(entity).queue(|mut entity: EntityWorldMut| {
                    facility::on_despawn(&mut entity);
                    entity.despawn();
                });
            }
            Some(TrackedId::Conduit(entity)) => {
                self.commands.entity(entity).queue(|mut entity: EntityWorldMut| {
                    conduit::on_despawn(&mut entity);
                    entity.despawn();
                });
            }
            None => tracing::error!("Received remove for unknown fixture id {:?}", fixture.id),
        }
    }
}

#[derive(SystemParam)]
struct SetFluidTypesParams<'w> {
    fluids: ResMut<'w, FluidTypes>,
}

impl UpdateHandler for SetFluidTypesParams<'_> {
    type Update = proto::SetFluidTypes;

    fn classify(_update: &Self::Update) -> HandlerClass { HandlerClass::Meta }

    fn handle(&mut self, update: &proto::SetFluidTypes) {
        self.fluids.0 = update.types.iter().map(|t| FluidType { name: t.name.clone() }).collect();
    }
}

#[derive(Component, Reflect)]
pub struct GenericViewable {
    pub name: String,
    pub kind: ViewableKind,
}

#[derive(Debug, Clone, Copy, Reflect)]
pub enum ViewableKind {
    Building,
    Corridor,
    Facility,
    Conduit,
}

#[derive(Resource, Default)]
pub struct FluidTypes(pub Vec<FluidType>);

pub struct FluidType {
    pub name: String,
}

#[derive(Config)]
pub struct Conf {
    subscription_level: view::SubscriptionLevel,
}
