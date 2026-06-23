use std::time::Duration;

use bevy::app::{self, App, Plugin};
use bevy::asset::{AssetServer, Assets};
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::name::Name;
use bevy::ecs::query::With;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Commands, ParamSet, Query, Res, ResMut, SystemParam, SystemState};
use bevy::ecs::world::EntityWorldMut;
use bevy::math::{Vec2, Vec3, Vec3Swizzles};
use bevy::picking::Pickable;
use bevy::reflect::Reflect;
use bevy::sprite_render::{ColorMaterial, MeshMaterial2d};
use bevy::time::{self, Time};
use bevy::transform::components::{GlobalTransform, Transform};
use bevy_mesh::Mesh2d;
use traffloat_physics::util::QueryExt;
use traffloat_proto::proto;

use crate::scene::picking::ObservePicking;
use crate::scene::{
    GenericViewable, HandlerClass, IdRegistry, ProtoId, TrackedId, UpdateHandler, ViewableKind,
    Zorder, building, corridor, facility,
};
use crate::util::shapes::Shapes;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<Info>();
        app.register_type::<DynamicPosition>();
        app.register_type::<Types>();
        app.init_resource::<Types>();
        app.add_systems(app::Update, update_dynamic_position_system);
    }
}

#[derive(Resource, Default, Reflect)]
pub struct Types {
    pub types:     Vec<proto::ResidentAttrType>,
    pub size_type: Option<usize>,
}

#[derive(SystemParam)]
pub(super) struct SetResidentAttrTypesParams<'w> {
    types: ResMut<'w, Types>,
}

impl UpdateHandler for SetResidentAttrTypesParams<'_> {
    type Update = proto::SetResidentAttrTypes;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Meta }

    fn handle(&mut self, update: &Self::Update) {
        self.types.types.clone_from(&update.types);
        self.types.size_type =
            update.types.iter().position(|ty| ty.niches.contains(proto::ResidentAttrNiche::SIZE));
    }
}

#[derive(SystemParam)]
pub(super) struct NewResidentParams<'w, 's> {
    commands:        Commands<'w, 's>,
    asset_server:    Res<'w, AssetServer>,
    materials:       ResMut<'w, Assets<ColorMaterial>>,
    shapes:          Shapes<'w>,
    ids_location_ps: ParamSet<'w, 's, (ResMut<'w, IdRegistry>, LocationResolver<'w, 's>)>,
}

impl UpdateHandler for NewResidentParams<'_, '_> {
    type Update = proto::NewResident;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Spawn }

    fn handle(&mut self, update: &Self::Update) {
        let proto_location = update.location.clone();
        let entity = self
            .commands
            .spawn((
                ProtoId(update.id),
                Name::new("Client resident"),
                GenericViewable { name: update.name.clone(), kind: ViewableKind::Resident },
                Mesh2d(self.shapes.square()),
                MeshMaterial2d(self.materials.add(ColorMaterial {
                    color: Color::WHITE,
                    texture: Some(self.asset_server.load("sprites/resident.png")),
                    ..Default::default()
                })),
                Pickable::default(),
            ))
            .queue(move |mut entity: EntityWorldMut| {
                let Some((location, dynamic_position)) = entity.world_scope(|world| {
                    let mut state = SystemState::<LocationResolver>::new(world);
                    let resolver = state.get_mut(world);
                    let resolve_result = resolver.resolve(&proto_location);
                    state.apply(world);
                    resolve_result
                }) else {
                    return;
                };
                entity.insert((Info { location, attributes: Vec::new() }, dynamic_position));
            })
            .observe_picking()
            .id();
        self.ids_location_ps.p0().map.insert(update.id, TrackedId::Resident(entity));
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
            proto::ResidentLocation::Facility { facility, ref slot_name } => {
                let entity = self.ids.get_facility(facility)?;
                let transform = self.facility_query.log_get(entity)?;
                (
                    Location::Facility { facility: entity, slot_name: slot_name.clone() },
                    transform.translation().xy(),
                    Vec2::ZERO,
                )
            }
        };
        Some((location, DynamicPosition { epoch_position, epoch_time: self.time.elapsed(), speed }))
    }
}

#[derive(SystemParam)]
pub(super) struct UpdateResidentAttributesFullParams<'w, 's> {
    ids:            Res<'w, IdRegistry>,
    types:          Res<'w, Types>,
    resident_query: Query<'w, 's, &'static mut Info>,
}

impl UpdateHandler for UpdateResidentAttributesFullParams<'_, '_> {
    type Update = proto::UpdateResidentAttributesFull;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Update }

    fn handle(&mut self, update: &Self::Update) {
        let Some(entity) = self.ids.get_resident(update.id) else { return };
        let Some(mut info) = self.resident_query.log_get_mut(entity) else { return };

        info.attributes.resize(self.types.types.len(), None);

        let mut attr_value_iter = update.attrs.iter();
        for (ty, slot) in self.types.types.iter().zip(&mut info.attributes) {
            if ty.subscribed.contains(update.sub) {
                let Some(&value) = attr_value_iter.next() else {
                    tracing::warn!(
                        "Received fewer resident attributes in full update than last received \
                         subscribed type definitions"
                    );
                    return;
                };
                *slot = Some(value);
            } else {
                *slot = None;
            }
        }

        if attr_value_iter.next().is_some() {
            tracing::warn!(
                "Received more resident attributes in full update than last received subscribed \
                 type definitions"
            );
        }
    }
}

#[derive(SystemParam)]
pub(super) struct UpdateResidentAttributesPartialParams<'w, 's> {
    ids:            Res<'w, IdRegistry>,
    types:          Res<'w, Types>,
    resident_query: Query<'w, 's, &'static mut Info>,
}

impl UpdateHandler for UpdateResidentAttributesPartialParams<'_, '_> {
    type Update = proto::UpdateResidentAttributesPartial;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Update }

    fn handle(&mut self, update: &Self::Update) {
        let Some(entity) = self.ids.get_resident(update.id) else { return };
        let Some(mut info) = self.resident_query.log_get_mut(entity) else { return };

        info.attributes.resize(self.types.types.len(), None);

        for &(ty, value) in &update.attrs {
            let ty = usize::try_from(ty).expect("usize >= u32 on supported targets");
            let Some(slot) = info.attributes.get_mut(ty) else {
                tracing::warn!(
                    "Received resident attribute update for undefined type index {ty} >= {}",
                    self.types.types.len()
                );
                continue;
            };
            *slot = Some(value);
        }
    }
}

#[derive(Component, Reflect)]
pub struct Info {
    pub location:   Location,
    pub attributes: Vec<Option<f32>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub enum Location {
    Building(Entity),
    Corridor(Entity),
    Facility { facility: Entity, slot_name: String },
}

#[derive(Component, Default, Reflect)]
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
    types: Res<Types>,
    resident_query: Query<(&mut Transform, &Info, &DynamicPosition)>,
) {
    for (mut transform, info, dynamic_position) in resident_query {
        let pos = dynamic_position.extrapolate(time.elapsed());
        transform.translation = pos.extend(Zorder::Resident.z());

        if let Some(size_type) = types.size_type
            && let Some(&Some(size)) = info.attributes.get(size_type)
        {
            transform.scale = Vec3::new(size, size, 1.0);
        }
    }
}

pub(super) fn on_despawn(_: &mut EntityWorldMut) {
    // reserved
}
