use std::time::Duration;

use bevy::app::{self, App, Plugin};
use bevy::asset::{AssetServer, Assets};
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::name::Name;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Query, Res, ResMut, SystemParam};
use bevy::ecs::world::EntityWorldMut;
use bevy::math::{Vec2, Vec3Swizzles};
use bevy::sprite_render::{ColorMaterial, MeshMaterial2d};
use bevy::time::{self, Time};
use bevy::transform::components::{GlobalTransform, Transform};
use bevy_mesh::Mesh2d;
use traffloat_physics::util::QueryExt;
use traffloat_proto::proto;

use crate::scene::{
    GenericViewable, HandlerClass, IdRegistry, TrackedId, UpdateHandler, ViewableKind, Zorder,
    building, corridor, facility,
};
use crate::util::shapes::Shapes;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) { app.add_systems(app::Update, update_dynamic_position_system); }
}

#[derive(SystemParam)]
pub(super) struct NewResidentParams<'w, 's> {
    commands:          Commands<'w, 's>,
    asset_server:      Res<'w, AssetServer>,
    materials:         ResMut<'w, Assets<ColorMaterial>>,
    shapes:            Shapes<'w>,
    location_resolver: LocationResolver<'w, 's>,
    ids:               ResMut<'w, IdRegistry>,
}

impl UpdateHandler for NewResidentParams<'_, '_> {
    type Update = proto::NewResident;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Spawn }

    fn handle(&mut self, update: &Self::Update) {
        let Some((location, dynamic_position)) = self.location_resolver.resolve(&update.location)
        else {
            return;
        };
        let entity = self
            .commands
            .spawn((
                Name::new("Client resident"),
                Info { location },
                dynamic_position,
                GenericViewable { name: update.name.clone(), kind: ViewableKind::Resident },
                Mesh2d(self.shapes.square()),
                MeshMaterial2d(self.materials.add(ColorMaterial {
                    color: Color::WHITE,
                    texture: Some(self.asset_server.load("sprites/resident.png")),
                    ..Default::default()
                })),
            ))
            .id();
        self.ids.map.insert(update.id, TrackedId::Resident(entity));
    }
}

#[derive(SystemParam)]
pub(super) struct UpdateResidentLocationParams<'w, 's> {
    ids:               Res<'w, IdRegistry>,
    resident_query:    Query<'w, 's, (&'static mut Info, &'static mut DynamicPosition)>,
    location_resolver: LocationResolver<'w, 's>,
}

impl UpdateHandler for UpdateResidentLocationParams<'_, '_> {
    type Update = proto::UpdateResidentLocation;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Update }

    fn handle(&mut self, update: &Self::Update) {
        let Some(entity) = self.ids.get_resident(update.id) else { return };
        let Some(mut data) = self.resident_query.log_get_mut(entity) else { return };
        let Some((location, dp)) = self.location_resolver.resolve(&update.location) else { return };
        data.0.location = location;
        *data.1 = dp;
    }
}

#[derive(SystemParam)]
struct LocationResolver<'w, 's> {
    ids:            Res<'w, IdRegistry>,
    building_query: Query<'w, 's, &'static building::Info>,
    corridor_query: Query<'w, 's, &'static corridor::Info>,
    facility_query: Query<'w, 's, &'static GlobalTransform, With<facility::Info>>,
    time:           Res<'w, Time<time::Virtual>>,
}

impl LocationResolver<'_, '_> {
    fn resolve(&self, location: &proto::ResidentLocation) -> Option<(Location, DynamicPosition)> {
        let (location, epoch_position, speed) = match *location {
            proto::ResidentLocation::Building { building, interior_pos, speed } => {
                let entity = self.ids.get_building(building)?;
                let building_info = self.building_query.log_get(entity)?;
                let position = building_info.position + interior_pos.xy();
                (Location::Building(entity), position, speed.xy())
            }
            proto::ResidentLocation::Corridor { corridor, linear_pos, speed } => {
                let entity = self.ids.get_corridor(corridor)?;
                let corridor_info = self.corridor_query.log_get(entity)?;
                let endpoints = corridor_info.endpoint_positions;
                let atob = endpoints.atob().normalize_or_zero();
                let position = endpoints.alpha + atob * linear_pos;
                (Location::Corridor(entity), position, atob * speed)
            }
            proto::ResidentLocation::Facility { facility } => {
                let entity = self.ids.get_facility(facility)?;
                let transform = self.facility_query.log_get(entity)?;
                (Location::Facility(entity), transform.translation().xy(), Vec2::ZERO)
            }
        };
        Some((location, DynamicPosition { epoch_position, epoch_time: self.time.elapsed(), speed }))
    }
}

#[derive(Component)]
pub struct Info {
    pub location: Location,
}

pub enum Location {
    Building(Entity),
    Corridor(Entity),
    Facility(Entity),
}

#[derive(Component, Default)]
pub struct DynamicPosition {
    epoch_position: Vec2,
    epoch_time:     Duration,
    speed:          Vec2,
}

impl DynamicPosition {
    fn extrapolate(&self, elapsed: Duration) -> Vec2 {
        let dt = elapsed.checked_sub(self.epoch_time).unwrap_or_default();
        self.epoch_position + self.speed * dt.as_secs_f32()
    }
}

fn update_dynamic_position_system(
    time: Res<Time<time::Virtual>>,
    query: Query<(&mut Transform, &DynamicPosition)>,
) {
    for (mut transform, dynamic_position) in query {
        let pos = dynamic_position.extrapolate(time.elapsed());
        transform.translation = pos.extend(Zorder::Resident.z());
    }
}

pub(super) fn on_despawn(_: &mut EntityWorldMut) {
    // reserved
}
