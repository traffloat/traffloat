//! A local mirror of the actual world based on incremental [proto](traffloat_proto) updates.

use std::collections::HashMap;
use std::mem;

use bevy::app::{self, App, Plugin};
use bevy::asset::Assets;
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::{Message, MessageReader};
use bevy::ecs::query::With;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Commands, ParamSet, Query, ResMut, Single, SystemParam};
use bevy::ecs::world::World;
use bevy::math::Vec3;
use bevy::mesh::Mesh2d;
use bevy::sprite_render::{ColorMaterial, MeshMaterial2d};
use bevy::transform::components::Transform;
use bevy_mod_config::{AppExt, Config, ReadConfig};
use traffloat_physics::util::QueryExt;
use traffloat_physics::{try_log, view};
use traffloat_proto::proto;

use crate::ConfigManager;

mod building;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.init_resource::<IdRegistry>();
        app.init_config::<ConfigManager, Conf>("scene");
        app.add_plugins(building::Plug);
        app.add_systems(app::Update, react_config_system);
        app.add_systems(app::Update, handle_update_system);
    }
}

#[derive(Resource, Default)]
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
                    mem::discriminant(v)
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
                    mem::discriminant(v)
                );
                None
            }
            None => {
                tracing::error!("Received unknown corridor id {id:?}");
                None
            }
        }
    }
}

enum TrackedId {
    Building(Entity),
    Corridor(Entity),
}

#[derive(Component)]
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
    Corridor,
    Facility,
    Conduit,
}

impl Zorder {
    #[expect(clippy::cast_precision_loss, reason = "COUNT < 2^(f32::MANTISSA_DIGITS)")]
    pub const fn z(self) -> f32 {
        0.25 + (self as u8 as f32) / (<Self as strum::EnumCount>::COUNT as f32) * 0.5
    }
}

fn handle_update_system(
    mut updates: MessageReader<view::SentUpdate>,
    mut params: HandleUpdateParams,
    viewer: Option<Single<Entity, With<SinglePlayerViewer>>>,
) {
    let Some(viewer) = viewer else { return };
    for update in updates.read() {
        if update.viewers.contains(&*viewer) {
            tracing::debug_span!("handle_update", update = ?update.body).in_scope(|| {
                params.handle(&update.body);
            });
        }
    }
}

macro_rules! define_params {
    (
        $w:lifetime, $s:lifetime;
        $(
            $short:ident($param:ty) ($($p1:ident)*),
        )*
    ) => {
        #[derive(SystemParam)]
        struct HandleUpdateParams<$w, $s>(recurse_nested_tuple!($w, $s; $($param,)*));

        impl HandleUpdateParams<'_, '_> {
            fn handle(&mut self, update: &proto::Update) {
                match update {
                    $(
                        proto::Update::$short(update) => self.0 $(.$p1())* .p0().handle(update),
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

define_params! {
    'w, 's;
    NewBuilding(building::NewBuildingParams<'w, 's>) (),
    UpdateBuilding(building::UpdateBuildingParams<'w, 's>) (p1),
    NewCorridor(NewCorridorParams<'w, 's>) (p1 p1),
    UpdateCorridor(UpdateCorridorParams<'w, 's>) (p1 p1 p1),
    RemoveViewable(RemoveViewableParams<'w, 's>) (p1 p1 p1 p1),
}

#[derive(SystemParam)]
struct NewCorridorParams<'w, 's> {
    commands: Commands<'w, 's>,
    ids:      ResMut<'w, IdRegistry>,
}

impl NewCorridorParams<'_, '_> {
    fn handle(&mut self, corridor: &proto::NewCorridor) {}
}

#[derive(SystemParam)]
struct UpdateCorridorParams<'w, 's> {
    commands: Commands<'w, 's>,
}

impl UpdateCorridorParams<'_, '_> {
    fn handle(&mut self, corridor: &proto::UpdateCorridor) {}
}

#[derive(SystemParam)]
struct RemoveViewableParams<'w, 's> {
    commands: Commands<'w, 's>,
    ids:      ResMut<'w, IdRegistry>,
}

impl RemoveViewableParams<'_, '_> {
    fn handle(&mut self, fixture: &proto::RemoveViewable) {
        match self.ids.map.remove(&fixture.id) {
            Some(TrackedId::Building(entity) | TrackedId::Corridor(entity)) => {
                self.commands.entity(entity).despawn();
            }
            None => tracing::error!("Received remove for unknown fixture id {:?}", fixture.id),
        }
    }
}

fn bevy_color(proto::Color([r, g, b, a]): proto::Color) -> Color { Color::linear_rgba(r, g, b, a) }

#[derive(Config)]
pub struct Conf {
    subscription_level: view::SubscriptionLevel,
}
