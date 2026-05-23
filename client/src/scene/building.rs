use std::collections::HashMap;
use std::mem;

use bevy::app::{self, App, Plugin};
use bevy::asset::{self, Assets};
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::message::{Message, MessageReader};
use bevy::ecs::query::With;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, ParamSet, Query, Res, ResMut, Single, SystemParam};
use bevy::ecs::world::World;
use bevy::math::Vec3;
use bevy::math::primitives::Annulus;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::sprite_render::{ColorMaterial, MeshMaterial2d};
use bevy::transform::components::Transform;
use bevy_mod_config::{AppExt, Config, ReadConfig};
use traffloat_physics::util::QueryExt;
use traffloat_physics::{try_log, view};
use traffloat_proto::proto;

use crate::ConfigManager;
use crate::scene::{IdRegistry, TrackedId, Zorder};
use crate::util::shapes::Shapes;

pub(super) struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.init_config::<ConfigManager, Conf>("scene:building");
        app.init_resource::<RingMaterial>();
        app.add_systems(app::Startup, RingMaterial::init);
        app.add_systems(app::Update, RingMaterial::update.ambiguous_with_all());
    }
}

#[derive(SystemParam)]
pub(super) struct NewBuildingParams<'w, 's> {
    commands:      Commands<'w, 's>,
    ids:           ResMut<'w, IdRegistry>,
    shapes:        Shapes<'w>,
    meshes:        ResMut<'w, Assets<Mesh>>,
    materials:     ResMut<'w, Assets<ColorMaterial>>,
    ring_material: Res<'w, RingMaterial>,
}

impl NewBuildingParams<'_, '_> {
    pub(super) fn handle(&mut self, update: &proto::NewBuilding) {
        let material = self.materials.add(ColorMaterial::from(Color::NONE));
        let entity = self
            .commands
            .spawn((
                super::ProtoId(update.id),
                Transform::from_translation(update.position.extend(Zorder::Building.z()))
                    .with_scale(Vec3::splat(update.radius)),
                Mesh2d(self.shapes.circle()),
                MeshMaterial2d(material),
            ))
            .with_related::<RingEntityOf>((
                Mesh2d(
                    self.meshes
                        .add(Annulus::new(update.radius, update.radius + update.wall_thickness)),
                ),
                MeshMaterial2d(self.ring_material.get().clone()),
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
struct RingMaterial(Option<asset::Handle<ColorMaterial>>);

impl RingMaterial {
    fn get(&self) -> &asset::Handle<ColorMaterial> {
        self.0.as_ref().expect("initialized during startup")
    }

    fn init(mut this: ResMut<Self>, mut materials: ResMut<Assets<ColorMaterial>>) {
        this.0 = Some(materials.add(ColorMaterial::from(Color::WHITE)));
    }

    fn update(
        this: Res<Self>,
        mut materials: ResMut<Assets<ColorMaterial>>,
        conf: ReadConfig<Conf>,
    ) {
        let material =
            materials.get_mut(this.get()).expect("ring material should reference a valid material");
        material.color = conf.read().wall_color;
    }
}

#[derive(SystemParam)]
pub(super) struct UpdateBuildingParams<'w, 's> {
    ids:            ResMut<'w, IdRegistry>,
    materials:      ResMut<'w, Assets<ColorMaterial>>,
    material_query: Query<'w, 's, &'static MeshMaterial2d<ColorMaterial>>,
}

impl UpdateBuildingParams<'_, '_> {
    pub(super) fn handle(&mut self, update: &proto::UpdateBuilding) {
        let Some(entity) = self.ids.get_building(update.id) else {
            tracing::error!("Received update for unknown building id {:?}", update.id);
            return;
        };
        let Ok(handle) = self.material_query.get(entity) else {
            // Happens when update is received immediately after update
            return;
        };
        let material = try_log!(self.materials.get_mut(&handle.0), expect "building entity should reference a valid material" or return);
        material.color = super::bevy_color(update.color);
    }
}

#[derive(Config)]
pub struct Conf {
    #[config(default = Color::WHITE)]
    pub wall_color: Color,
}
