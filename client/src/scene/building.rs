use std::collections::HashMap;
use std::mem;

use bevy::app::{self, App, Plugin};
use bevy::asset::{self, Assets};
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::{Message, MessageReader};
use bevy::ecs::observer;
use bevy::ecs::query::{Has, With};
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, ParamSet, Query, Res, ResMut, Single, SystemParam};
use bevy::ecs::world::World;
use bevy::math::Vec3;
use bevy::math::primitives::Annulus;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::picking::{Pickable, events as pick};
use bevy::sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d};
use bevy::transform::components::Transform;
use bevy_mod_config::{AppExt, Config, ReadConfig};
use traffloat_physics::util::QueryExt;
use traffloat_physics::{try_log, view};
use traffloat_proto::proto;

use crate::ConfigManager;
use crate::scene::picking::{self, ObservePicking};
use crate::scene::{GenericViewable, IdRegistry, TrackedId, ViewableKind, Zorder};
use crate::util::shapes::Shapes;

pub(super) struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.init_config::<ConfigManager, Conf>("scene:building");
        app.init_resource::<RingMaterials>();
        app.add_systems(app::Startup, RingMaterials::init);
        app.add_systems(app::Update, RingMaterials::update.ambiguous_with_all());
        app.add_systems(app::Update, update_ring_hover_system);
    }
}

#[derive(SystemParam)]
pub(super) struct NewBuildingParams<'w, 's> {
    commands:       Commands<'w, 's>,
    ids:            ResMut<'w, IdRegistry>,
    shapes:         Shapes<'w>,
    meshes:         ResMut<'w, Assets<Mesh>>,
    materials:      ResMut<'w, Assets<ColorMaterial>>,
    ring_materials: Res<'w, RingMaterials>,
}

impl NewBuildingParams<'_, '_> {
    pub(super) fn handle(&mut self, update: &proto::NewBuilding) {
        let material = self.materials.add(ColorMaterial {
            color: Color::NONE,
            alpha_mode: AlphaMode2d::Blend,
            ..Default::default()
        });
        let entity = self
            .commands
            .spawn((
                super::ProtoId(update.id),
                Transform::from_translation(update.position.extend(Zorder::Building.z()))
                    .with_scale(Vec3::splat(update.radius)),
                Mesh2d(self.shapes.circle()),
                MeshMaterial2d(material),
                Pickable::default(),
                GenericViewable { name: update.name.clone(), kind: ViewableKind::Building },
                Info::default(),
            ))
            .observe_picking()
            .with_related::<RingEntityOf>((
                Mesh2d(
                    self.meshes
                        .add(Annulus::new(update.radius, update.radius + update.wall_thickness)),
                ),
                MeshMaterial2d(self.ring_materials.get_base().clone()),
                Transform::from_translation(update.position.extend(Zorder::Building.z())),
            ))
            .id();
        self.ids.map.insert(update.id, TrackedId::Building(entity));
    }
}

#[derive(Component)]
#[relationship(relationship_target = HasRingEntity)]
struct RingEntityOf(Entity);

#[derive(Component)]
#[relationship_target(relationship =RingEntityOf)]
struct HasRingEntity(Entity);

#[derive(Resource, Default)]
struct RingMaterials {
    base:    Option<asset::Handle<ColorMaterial>>,
    hovered: Option<asset::Handle<ColorMaterial>>,
}

impl RingMaterials {
    fn get_base(&self) -> &asset::Handle<ColorMaterial> {
        self.base.as_ref().expect("initialized during startup")
    }

    fn get_hovered(&self) -> &asset::Handle<ColorMaterial> {
        self.hovered.as_ref().expect("initialized during startup")
    }

    fn init(mut this: ResMut<Self>, mut materials: ResMut<Assets<ColorMaterial>>) {
        this.base = Some(materials.add(ColorMaterial::from(Color::WHITE)));
        this.hovered = Some(materials.add(ColorMaterial::from(Color::WHITE)));
    }

    fn update(
        this: Res<Self>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        conf: ReadConfig<Conf>,
    ) {
        materials
            .get_mut(this.get_base())
            .expect("ring material should reference a valid material")
            .color = conf.read().wall_color;
        materials
            .get_mut(this.get_hovered())
            .expect("ring material should reference a valid material")
            .color = conf.read().hovered_wall_color;
    }
}

#[derive(SystemParam)]
pub(super) struct UpdateBuildingParams<'w, 's> {
    ids:            ResMut<'w, IdRegistry>,
    materials:      ResMut<'w, Assets<ColorMaterial>>,
    building_query: Query<'w, 's, (&'static MeshMaterial2d<ColorMaterial>, &'static mut Info)>,
}

impl UpdateBuildingParams<'_, '_> {
    pub(super) fn handle(&mut self, update: &proto::UpdateBuilding) {
        let Some(entity) = self.ids.get_building(update.id) else {
            tracing::error!("Received update for unknown building id {:?}", update.id);
            return;
        };
        let Ok((handle, mut info)) = self.building_query.get_mut(entity) else {
            // Happens when update is received immediately after update
            return;
        };
        let material = try_log!(self.materials.get_mut(&handle.0), expect "building entity should reference a valid material" or return);
        material.color = super::bevy_color(update.color);

        info.ambient_fluid = None;
    }
}

#[derive(SystemParam)]
pub(super) struct UpdateBuildingFullParams<'w, 's> {
    ids:            ResMut<'w, IdRegistry>,
    materials:      ResMut<'w, Assets<ColorMaterial>>,
    building_query: Query<'w, 's, (&'static MeshMaterial2d<ColorMaterial>, &'static mut Info)>,
}

impl UpdateBuildingFullParams<'_, '_> {
    pub(super) fn handle(&mut self, update: &proto::UpdateBuildingFull) {
        let Some(entity) = self.ids.get_building(update.id) else {
            tracing::error!("Received update for unknown building id {:?}", update.id);
            return;
        };
        let Ok((handle, mut info)) = self.building_query.get_mut(entity) else {
            // Happens when update is received immediately after update
            return;
        };
        let material = try_log!(self.materials.get_mut(&handle.0), expect "building entity should reference a valid material" or return);
        material.color = super::bevy_color(update.color);

        info.ambient_fluid = Some(update.ambient_fluid.clone());
    }
}

#[derive(Default, Component)]
pub struct Info {
    pub ambient_fluid: Option<proto::FluidStorageFull>,
}

fn update_ring_hover_system(
    ring_query: Query<(&mut MeshMaterial2d<ColorMaterial>, &RingEntityOf)>,
    building_query: Query<Has<picking::Hovered>>,
    ring_materials: Res<RingMaterials>,
) {
    for (mut material, parent) in ring_query {
        let hovered = building_query.get(parent.0).unwrap_or(false);
        let desired =
            if hovered { ring_materials.get_hovered() } else { ring_materials.get_base() };
        if material.0 != *desired {
            material.0 = desired.clone();
        }
    }
}

#[derive(Config)]
pub struct Conf {
    #[config(default = Color::WHITE)]
    pub wall_color:         Color,
    #[config(default = Color::srgb(1.0, 0.4, 0.5))]
    pub hovered_wall_color: Color,
}
