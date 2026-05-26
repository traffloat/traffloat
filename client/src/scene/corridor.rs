use std::collections::HashMap;
use std::mem;

use bevy::app::{self, App, Plugin};
use bevy::asset::{self, Assets};
use bevy::color::Color;
use bevy::ecs::bundle::Bundle;
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
        app.init_config::<ConfigManager, Conf>("scene:corridor");
        app.init_resource::<WallMaterials>();
        app.add_systems(app::Startup, WallMaterials::init);
        app.add_systems(app::Update, WallMaterials::update.ambiguous_with_all());
        app.add_systems(app::Update, update_wall_hover_system::<true>);
        app.add_systems(app::Update, update_wall_hover_system::<false>);
    }
}

#[derive(SystemParam)]
pub(super) struct NewCorridorParams<'w, 's> {
    commands:       Commands<'w, 's>,
    ids:            ResMut<'w, IdRegistry>,
    shapes:         Shapes<'w>,
    meshes:         ResMut<'w, Assets<Mesh>>,
    materials:      ResMut<'w, Assets<ColorMaterial>>,
    wall_materials: Res<'w, WallMaterials>,
}

impl NewCorridorParams<'_, '_> {
    pub(super) fn handle(&mut self, update: &proto::NewCorridor) {
        fn wall_rect<const WHICH: bool>(
            shapes: &Shapes,
            update: &proto::NewCorridor,
        ) -> impl Bundle {
            let delta = update.alpha_position - update.beta_position;
            let mut offset =
                delta.perp().normalize_or_zero() * (update.radius + update.wall_thickness * 0.5);
            if WHICH {
                offset *= -1.0;
            }
            shapes.rect(
                update.wall_thickness,
                update.alpha_position + offset,
                update.beta_position + offset,
                Zorder::CorridorWall,
            )
        }
        let material = self.materials.add(ColorMaterial {
            color: Color::NONE,
            alpha_mode: AlphaMode2d::Blend,
            ..Default::default()
        });
        let entity = self
            .commands
            .spawn((
                super::ProtoId(update.id),
                self.shapes.rect(
                    update.radius * 2.0,
                    update.alpha_position,
                    update.beta_position,
                    Zorder::Corridor,
                ),
                MeshMaterial2d(material),
                Pickable::default(),
                GenericViewable { name: update.name.clone(), kind: ViewableKind::Corridor },
                Info::default(),
            ))
            .observe_picking()
            .with_related::<WallEntityOf<true>>((
                MeshMaterial2d(self.wall_materials.get_base().clone()),
                wall_rect::<true>(&self.shapes, update),
            ))
            .with_related::<WallEntityOf<false>>((
                MeshMaterial2d(self.wall_materials.get_base().clone()),
                wall_rect::<false>(&self.shapes, update),
            ))
            .id();
    }
}

#[derive(SystemParam)]
pub(super) struct UpdateCorridorParams<'w, 's> {
    commands: Commands<'w, 's>,
}

impl UpdateCorridorParams<'_, '_> {
    pub(super) fn handle(&mut self, corridor: &proto::UpdateCorridor) {}
}

#[derive(SystemParam)]
pub(super) struct UpdateCorridorFullParams<'w, 's> {
    commands: Commands<'w, 's>,
}

impl UpdateCorridorFullParams<'_, '_> {
    pub(super) fn handle(&mut self, corridor: &proto::UpdateCorridorFull) {}
}

#[derive(Default, Component)]
pub struct Info {
    pub ambient_fluid: Option<proto::FluidStorageFull>,
}

#[derive(Component)]
#[relationship(relationship_target = HasWallEntity<WHICH>)]
struct WallEntityOf<const WHICH: bool>(Entity);

#[derive(Component)]
#[relationship_target(relationship =WallEntityOf<WHICH>)]
struct HasWallEntity<const WHICH: bool>(Entity);

#[derive(Resource, Default)]
struct WallMaterials {
    base:    Option<asset::Handle<ColorMaterial>>,
    hovered: Option<asset::Handle<ColorMaterial>>,
}

impl WallMaterials {
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
            .expect("wall material should reference a valid material")
            .color = conf.read().wall_color;
        materials
            .get_mut(this.get_hovered())
            .expect("wall material should reference a valid material")
            .color = conf.read().hovered_wall_color;
    }
}

fn update_wall_hover_system<const WHICH: bool>(
    wall_query: Query<(&mut MeshMaterial2d<ColorMaterial>, &WallEntityOf<WHICH>)>,
    corridor_query: Query<Has<picking::Hovered>>,
    wall_materials: Res<WallMaterials>,
) {
    for (mut material, parent) in wall_query {
        let hovered = corridor_query.get(parent.0).unwrap_or(false);
        let desired =
            if hovered { wall_materials.get_hovered() } else { wall_materials.get_base() };
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
