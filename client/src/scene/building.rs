use bevy::app::{self, App, Plugin};
use bevy::asset::{self, Assets};
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::entity::{Entity, EntityHashSet};
use bevy::ecs::name::Name;
use bevy::ecs::query::Has;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, Query, Res, ResMut, SystemParam};
use bevy::math::primitives::Annulus;
use bevy::math::{Vec2, Vec3};
use bevy::mesh::{Mesh, Mesh2d};
use bevy::picking::Pickable;
use bevy::reflect::Reflect;
use bevy::sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d};
use bevy::transform::components::Transform;
use bevy_mod_config::{AppExt, Config, ReadConfig};
use traffloat_physics::try_log;
use traffloat_physics::util::QueryExt;
use traffloat_proto::proto;

use crate::scene::facility::FacilityBuilding;
use crate::scene::picking::{self, ObservePicking};
use crate::scene::{
    GenericViewable, HandlerClass, IdRegistry, TrackedId, UpdateHandler, ViewableKind, Zorder,
};
use crate::util::shapes::Shapes;
use crate::{ConfigManager, dock};

pub(super) struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<WallMaterials>();
        app.register_type::<Info>();
        app.register_type::<WallEntityOf>();
        app.register_type::<WallMaterials>();

        app.init_config::<ConfigManager, Conf>("scene:building");
        app.init_resource::<WallMaterials>();
        app.add_systems(app::Startup, WallMaterials::init);
        app.add_systems(app::Update, WallMaterials::update.ambiguous_with_all());
        app.add_systems(app::Update, update_wall_hover_system);
    }
}

#[derive(SystemParam)]
pub(super) struct NewBuildingParams<'w, 's> {
    commands:       Commands<'w, 's>,
    ids:            ResMut<'w, IdRegistry>,
    shapes:         Shapes<'w>,
    meshes:         ResMut<'w, Assets<Mesh>>,
    materials:      ResMut<'w, Assets<ColorMaterial>>,
    wall_materials: Res<'w, WallMaterials>,
}

impl UpdateHandler for NewBuildingParams<'_, '_> {
    type Update = proto::NewBuilding;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Spawn }

    fn handle(&mut self, update: &proto::NewBuilding) {
        let material = self.materials.add(ColorMaterial {
            color: Color::NONE,
            alpha_mode: AlphaMode2d::Blend,
            ..Default::default()
        });
        let entity = self
            .commands
            .spawn((
                Name::new("Client building"),
                super::ProtoId(update.id),
                Transform::from_translation(update.position.extend(Zorder::Building.z()))
                    .with_scale(Vec3::splat(update.radius)),
                Mesh2d(self.shapes.circle()),
                MeshMaterial2d(material),
                Pickable::default(),
                GenericViewable { name: update.name.clone(), kind: ViewableKind::Building },
                Info { position: update.position, radius: update.radius, ..Default::default() },
            ))
            .observe_picking()
            .with_related::<WallEntityOf>((
                Mesh2d(
                    self.meshes
                        .add(Annulus::new(update.radius, update.radius + update.wall_thickness)),
                ),
                MeshMaterial2d(self.wall_materials.get_base().clone()),
                Transform::from_translation(update.position.extend(Zorder::BuildingWall.z())),
            ))
            .id();
        self.ids.map.insert(update.id, TrackedId::Building(entity));
    }
}

#[derive(SystemParam)]
pub(super) struct UpdateBuildingParams<'w, 's> {
    ids:            ResMut<'w, IdRegistry>,
    materials:      ResMut<'w, Assets<ColorMaterial>>,
    building_query: Query<'w, 's, (&'static MeshMaterial2d<ColorMaterial>, &'static mut Info)>,
}

impl UpdateHandler for UpdateBuildingParams<'_, '_> {
    type Update = proto::UpdateBuilding;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Update }

    fn handle(&mut self, update: &proto::UpdateBuilding) {
        let Some(entity) = self.ids.get_building(update.id) else { return };
        let Ok((handle, mut info)) = self.building_query.get_mut(entity) else {
            // Happens when update is received immediately after update
            return;
        };
        let material = try_log!(self.materials.get_mut(&handle.0), expect "building entity should reference a valid material" or return);
        material.color = update.color.into();

        info.ambient_fluid = None;
    }
}

#[derive(SystemParam)]
pub(super) struct UpdateBuildingFullParams<'w, 's> {
    ids:            ResMut<'w, IdRegistry>,
    materials:      ResMut<'w, Assets<ColorMaterial>>,
    building_query: Query<'w, 's, (&'static MeshMaterial2d<ColorMaterial>, &'static mut Info)>,
}

impl UpdateHandler for UpdateBuildingFullParams<'_, '_> {
    type Update = proto::UpdateBuildingFull;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Update }

    fn handle(&mut self, update: &proto::UpdateBuildingFull) {
        let Some(entity) = self.ids.get_building(update.id) else { return };
        let Ok((handle, mut info)) = self.building_query.get_mut(entity) else {
            // Happens when update is received immediately after update
            return;
        };
        let material = try_log!(self.materials.get_mut(&handle.0), expect "building entity should reference a valid material" or return);
        material.color = update.color.into();

        info.ambient_fluid = Some(update.ambient_fluid.clone());
    }
}

#[derive(SystemParam)]
pub struct SetBuildingFluidConnectionsParams<'w, 's> {
    ids:            ResMut<'w, IdRegistry>,
    building_query: Query<'w, 's, &'static mut Info>,
}

impl UpdateHandler for SetBuildingFluidConnectionsParams<'_, '_> {
    type Update = proto::SetBuildingFluidConnections;

    fn classify(_update: &Self::Update) -> HandlerClass { HandlerClass::Update }

    fn handle(&mut self, update: &Self::Update) {
        let Some(entity) = self.ids.get_building(update.id) else { return };
        let Some(mut info) = self.building_query.log_get_mut(entity) else { return };
        info.connections.clone_from(&update.connections);
    }
}

#[derive(Default, Component, Reflect)]
pub struct Info {
    pub position:      Vec2,
    pub radius:        f32,
    pub ambient_fluid: Option<proto::FluidStorageFull>,
    pub connections:   Vec<proto::BuildingFluidConnection>,
}

impl Info {
    pub fn facility_fluid_connections(
        &self,
        target_facility: proto::Id,
        ids: &IdRegistry,
    ) -> impl Iterator<Item = (&proto::BuildingFluidConnection, FluidConnectionPeer)> {
        self.connections.iter().filter_map(move |conn| {
            let peer = match conn.pair {
                proto::BuildingFluidConnectionPair::FacilityFacility(a, b) => (a
                    == target_facility)
                    .then_some(b)
                    .or_else(|| (b == target_facility).then_some(a))
                    .and_then(|facility| ids.get_facility(facility))
                    .map(FluidConnectionPeer::Facility),
                proto::BuildingFluidConnectionPair::FacilityBuilding { facility, building } => {
                    (building == target_facility)
                        .then(|| ids.get_building(building))
                        .flatten()
                        .map(FluidConnectionPeer::Building)
                }
                proto::BuildingFluidConnectionPair::FacilityPipe { facility, pipe } => (facility
                    == target_facility)
                    .then(|| ids.get_conduit(pipe))
                    .flatten()
                    .map(FluidConnectionPeer::Pipe),
            };
            peer.map(|peer| (conn, peer))
        })
    }
}

pub enum FluidConnectionPeer {
    Facility(Entity),
    Building(Entity),
    Pipe(Entity),
}

#[derive(Component, Resource, Reflect)]
#[relationship(relationship_target = HasWallEntity)]
struct WallEntityOf(Entity);

#[derive(Component, Resource, Reflect)]
#[relationship_target(relationship = WallEntityOf)]
struct HasWallEntity(Entity);

#[derive(Resource, Reflect, Default)]
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

fn update_wall_hover_system(
    wall_query: Query<(&mut MeshMaterial2d<ColorMaterial>, &WallEntityOf)>,
    building_query: Query<Has<picking::Hovered>>,
    wall_materials: Res<WallMaterials>,
) {
    for (mut material, parent) in wall_query {
        let hovered = building_query.get(parent.0).unwrap_or(false);
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

fn sync_clicked_pickable(
    dock: &dock::State,
    facility_query: Query<(&mut Pickable, &FacilityBuilding)>,
) {
    let opened_entities: EntityHashSet = dock
        .tabs()
        .filter_map(|tab| match tab {
            dock::TabEnum::ViewableInfo(tab) => Some(tab.entity),
            _ => None,
        })
        .collect();

    for (mut pickable, building) in facility_query {
        *pickable = if opened_entities.contains(&building.0) {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
    }
}
