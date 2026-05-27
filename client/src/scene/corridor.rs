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
use bevy::ecs::query::{Has, QueryData, With};
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, ParamSet, Query, Res, ResMut, Single, SystemParam};
use bevy::ecs::world::{Mut, World};
use bevy::math::Vec3;
use bevy::math::primitives::Annulus;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::picking::{Pickable, events as pick};
use bevy::sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d};
use bevy::transform::components::Transform;
use bevy_mod_config::{AppExt, Config, ReadConfig};
use traffloat_physics::util::{Alpha, Beta, QueryExt, Which};
use traffloat_physics::{try_log, view};
use traffloat_proto::proto;

use crate::ConfigManager;
use crate::scene::picking::{self, ObservePicking};
use crate::scene::{
    GenericViewable, HandlerClass, IdRegistry, TrackedId, UpdateHandler, ViewableKind, Zorder,
};
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

impl UpdateHandler for NewCorridorParams<'_, '_> {
    type Update = proto::NewCorridor;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Spawn }

    fn handle(&mut self, update: &proto::NewCorridor) {
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
        self.ids.map.insert(update.id, TrackedId::Corridor(entity));
    }
}

#[derive(SystemParam)]
pub(super) struct UpdateCorridorParams<'w, 's> {
    ids:            ResMut<'w, IdRegistry>,
    materials:      ResMut<'w, Assets<ColorMaterial>>,
    corridor_query: Query<'w, 's, (&'static MeshMaterial2d<ColorMaterial>, &'static mut Info)>,
}

impl UpdateHandler for UpdateCorridorParams<'_, '_> {
    type Update = proto::UpdateCorridor;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Update }

    fn handle(&mut self, update: &proto::UpdateCorridor) {
        let Some(entity) = self.ids.get_corridor(update.id) else {
            tracing::error!("Received update for unknown corridor id {:?}", update.id);
            return;
        };
        let Ok((handle, mut info)) = self.corridor_query.get_mut(entity) else {
            // Happens when update is received immediately after update
            return;
        };
        let material = try_log!(self.materials.get_mut(&handle.0), expect "corridor entity should reference a valid material" or return);
        material.color = super::bevy_color(update.color);

        info.ambient_fluid = None;
    }
}

#[derive(SystemParam)]
pub(super) struct UpdateCorridorFullParams<'w, 's> {
    ids:            ResMut<'w, IdRegistry>,
    materials:      ResMut<'w, Assets<ColorMaterial>>,
    corridor_query: Query<'w, 's, (&'static MeshMaterial2d<ColorMaterial>, &'static mut Info)>,
}

impl UpdateHandler for UpdateCorridorFullParams<'_, '_> {
    type Update = proto::UpdateCorridorFull;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Update }

    fn handle(&mut self, update: &proto::UpdateCorridorFull) {
        let Some(entity) = self.ids.get_corridor(update.id) else {
            tracing::error!("Received update for unknown corridor id {:?}", update.id);
            return;
        };
        let Ok((handle, mut info)) = self.corridor_query.get_mut(entity) else {
            // Happens when update is received immediately after update
            return;
        };
        let material = try_log!(self.materials.get_mut(&handle.0), expect "corridor entity should reference a valid material" or return);
        material.color = super::bevy_color(update.color);

        info.ambient_fluid = Some(update.ambient_fluid.clone());
    }
}

#[derive(SystemParam)]
pub(super) struct SetCorridorEndpointParams<'w, 's> {
    ids:            ResMut<'w, IdRegistry>,
    corridor_query: Query<'w, 's, CorridorEndpointQueryData>,
    commands:       Commands<'w, 's>,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct CorridorEndpointQueryData {
    alpha: Option<(&'static EndpointRef<Alpha>, &'static mut GenericEndpointDetails<Alpha>)>,
    beta:  Option<(&'static EndpointRef<Beta>, &'static mut GenericEndpointDetails<Beta>)>,
}

impl UpdateHandler for SetCorridorEndpointParams<'_, '_> {
    type Update = proto::SetCorridorEndpoint;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::MixedSpawn }

    fn handle(&mut self, update: &proto::SetCorridorEndpoint) {
        let Some(corridor) = self.ids.get_corridor(update.corridor) else {
            tracing::error!("Received update for unknown corridor id {:?}", update.corridor);
            return;
        };

        let Some(data) = self.corridor_query.log_get_mut(corridor) else { return };

        let current = match update.which {
            proto::AlphaOrBeta::Alpha => {
                data.alpha.map(|(b, d)| (b.0, Mut::map_unchanged(d, |d| &mut d.0)))
            }
            proto::AlphaOrBeta::Beta => {
                data.beta.map(|(b, d)| (b.0, Mut::map_unchanged(d, |d| &mut d.0)))
            }
        };

        match (current, &update.value) {
            (None, None) => {}
            (None, Some(value)) => {
                let building = try_log!(
                    self.ids.get_building(value.building),
                    expect "Received update to conncet corridor {:?} to unknown building {:?}"
                    (update.corridor, value.building)
                    or return
                );
                let details = EndpointDetails::from(value);

                let mut ec = self.commands.entity(corridor);
                _ = match update.which {
                    proto::AlphaOrBeta::Alpha => ec.insert((
                        EndpointRef(building, Alpha),
                        GenericEndpointDetails(details, Alpha),
                    )),
                    proto::AlphaOrBeta::Beta => ec.insert((
                        EndpointRef(building, Beta),
                        GenericEndpointDetails(details, Beta),
                    )),
                };
            }
            (Some(_), None) => {
                let mut ec = self.commands.entity(corridor);
                _ = match update.which {
                    proto::AlphaOrBeta::Alpha => {
                        ec.remove::<(EndpointRef<Alpha>, GenericEndpointDetails<Alpha>)>()
                    }
                    proto::AlphaOrBeta::Beta => {
                        ec.remove::<(EndpointRef<Beta>, GenericEndpointDetails<Beta>)>()
                    }
                };
            }
            (Some((curr_building, mut curr_detail)), Some(value)) => {
                let building = try_log!(
                    self.ids.get_building(value.building),
                    expect "Received update to conncet corridor {:?} to unknown building {:?}"
                    (update.corridor, value.building)
                    or return
                );
                if curr_building != building {
                    let mut ec = self.commands.entity(corridor);
                    _ = match update.which {
                        proto::AlphaOrBeta::Alpha => ec.insert((EndpointRef(building, Alpha),)),
                        proto::AlphaOrBeta::Beta => ec.insert((EndpointRef(building, Beta),)),
                    };
                }
                *curr_detail = EndpointDetails::from(value);
            }
        }
    }
}

#[derive(Default, Component)]
pub struct Info {
    pub ambient_fluid: Option<proto::FluidStorageFull>,
}

/// References building from corridor.
#[derive(Component)]
#[relationship(relationship_target = IsEndpointOf<Ab>)]
pub struct EndpointRef<Ab: Which>(#[relationship] pub Entity, Ab);

#[derive(Component)]
pub struct GenericEndpointDetails<Ab: Which>(pub EndpointDetails, Ab);

/// References corridor from building.
#[derive(Component)]
#[relationship_target(relationship = EndpointRef<Ab>)]
pub struct IsEndpointOf<Ab: Which>(#[relationship] Vec<Entity>, Ab);

pub struct EndpointDetails {
    pub open: bool,
}

impl From<&proto::CorridorEndpoint> for EndpointDetails {
    fn from(value: &proto::CorridorEndpoint) -> Self { Self { open: value.open } }
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
