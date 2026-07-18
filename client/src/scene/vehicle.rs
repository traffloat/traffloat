use std::iter;
use std::time::Duration;

use bevy::app::{self, App, Plugin};
use bevy::asset::{self, AssetServer, Assets};
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::name::Name;
use bevy::ecs::query::With;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Commands, ParamSet, Query, Res, ResMut, SystemParam};
use bevy::ecs::world::EntityWorldMut;
use bevy::image::Image;
use bevy::math::{Vec2, Vec3, Vec3Swizzles};
use bevy::picking::Pickable;
use bevy::reflect::Reflect;
use bevy::sprite_render::{ColorMaterial, MeshMaterial2d};
use bevy::time::{self, Time};
use bevy::transform::components::{GlobalTransform, Transform};
use bevy_mesh::Mesh2d;
use traffloat_physics::util::{QueryExt, run_stateless_closure};
use traffloat_proto::proto;

use crate::scene::conduit::ConduitCorridor;
use crate::scene::picking::ObservePicking;
use crate::scene::{
    GenericViewable, HandlerClass, IdRegistry, ProtoId, TrackedId, UpdateHandler, ViewableKind,
    Zorder, building, conduit, corridor, facility,
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
    pub types: Vec<Type>,
}

#[derive(Reflect)]
pub struct Type {
    pub proto:  proto::VehicleType,
    pub sprite: asset::Handle<Image>,
}

#[derive(SystemParam)]
pub(super) struct SetVehicleTypesParams<'w> {
    types:        ResMut<'w, Types>,
    asset_server: Res<'w, AssetServer>,
}

impl UpdateHandler for SetVehicleTypesParams<'_> {
    type Update = proto::SetVehicleTypes;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Meta }

    fn handle(&mut self, update: &Self::Update) {
        self.types.types.clear();
        self.types.types.extend(update.types.iter().map(|ty| Type {
            proto:  ty.clone(),
            sprite: self.asset_server.load(format!("sprites/{}.png", ty.sprite_id)),
        }));
    }
}

#[derive(SystemParam)]
pub(super) struct NewVehicleParams<'w, 's> {
    commands:        Commands<'w, 's>,
    types:           Res<'w, Types>,
    materials:       ResMut<'w, Assets<ColorMaterial>>,
    shapes:          Shapes<'w>,
    ids_location_ps: ParamSet<'w, 's, (ResMut<'w, IdRegistry>, LocationResolver<'w, 's>)>,
}

impl UpdateHandler for NewVehicleParams<'_, '_> {
    type Update = proto::NewVehicle;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Spawn }

    fn handle(&mut self, update: &Self::Update) {
        let type_id = usize::try_from(update.ty).expect("usize >= u32 on supported targets");
        let Some(ty) = self.types.types.get(type_id) else {
            tracing::warn!(
                "Received new vehicle update for undefined type index {type_id} >= {}",
                self.types.types.len()
            );
            return;
        };
        let num_compartments = ty.proto.compartments.len();

        let proto_location = update.location.clone();
        let entity = self
            .commands
            .spawn((
                ProtoId(update.id),
                Name::new("Client vehicle"),
                GenericViewable { name: update.name.clone(), kind: ViewableKind::Vehicle },
                Mesh2d(self.shapes.square()),
                MeshMaterial2d(self.materials.add(ColorMaterial {
                    color: Color::WHITE,
                    texture: Some(ty.sprite.clone()),
                    ..Default::default()
                })),
                Transform::from_scale((ty.proto.sprite_scale, 1.0).into()),
                Pickable::default(),
            ))
            .queue(move |mut entity: EntityWorldMut| {
                let Some((location, dynamic_position)) = entity.world_scope(|world| {
                    run_stateless_closure(world, move |resolver: LocationResolver<'_, '_>| {
                        resolver.resolve(&proto_location)
                    })
                }) else {
                    return;
                };
                entity.insert((
                    Info {
                        location,
                        ty: type_id,
                        compartments: iter::repeat_with(CompartmentInfo::default)
                            .take(num_compartments)
                            .collect(),
                    },
                    dynamic_position,
                ));
            })
            .observe_picking()
            .id();
        self.ids_location_ps.p0().map.insert(update.id, TrackedId::Vehicle(entity));
    }
}

#[derive(SystemParam)]
pub(super) struct UpdateVehicleLocationParams<'w, 's> {
    ids:               Res<'w, IdRegistry>,
    vehicle_query:     Query<'w, 's, (&'static mut Info, &'static mut DynamicPosition)>,
    location_resolver: LocationResolver<'w, 's>,
}

impl UpdateHandler for UpdateVehicleLocationParams<'_, '_> {
    type Update = proto::UpdateVehicleLocation;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Update }

    fn handle(&mut self, update: &Self::Update) {
        let Some(entity) = self.ids.get_vehicle(update.id) else { return };
        let Some(mut data) = self.vehicle_query.log_get_mut(entity) else { return };
        let Some((location, dp)) = self.location_resolver.resolve(&update.location) else { return };
        data.0.location = location;
        *data.1 = dp;
    }
}

#[derive(SystemParam)]
pub(super) struct UpdateVehicleFluidParams<'w, 's> {
    ids:           Res<'w, IdRegistry>,
    vehicle_query: Query<'w, 's, (&'static MeshMaterial2d<ColorMaterial>, &'static mut Info)>,
    materials:     ResMut<'w, Assets<ColorMaterial>>,
}

impl UpdateHandler for UpdateVehicleFluidParams<'_, '_> {
    type Update = proto::UpdateVehicleFluid;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Update }

    fn handle(&mut self, update: &Self::Update) {
        let Some(entity) = self.ids.get_vehicle(update.id) else { return };
        let Some((material_handle, mut info)) = self.vehicle_query.log_get_mut(entity) else {
            return;
        };

        let mut material =
            self.materials.get_mut(&material_handle.0).expect("strong handle must be valid");
        material.color = update.taint.into();

        if let Some(update_compartments) = &update.compartments {
            for (cpmt_index, (fluid_detail, cpmt_info)) in
                update_compartments.iter().zip(&mut info.compartments).enumerate()
            {
                cpmt_info.fluid = Some(fluid_detail.clone());
            }
        } else {
            for cpmt_info in &mut info.compartments {
                cpmt_info.fluid = None;
            }
        }
    }
}

#[derive(SystemParam)]
struct LocationResolver<'w, 's> {
    ids:            Res<'w, IdRegistry>,
    building_query: Query<'w, 's, &'static building::Info>,
    conduit_query:  Query<'w, 's, &'static ConduitCorridor>,
    corridor_query: Query<'w, 's, &'static corridor::Info>,
    time:           Res<'w, Time<time::Virtual>>,
}

impl LocationResolver<'_, '_> {
    fn resolve(&self, location: &proto::VehicleLocation) -> Option<(Location, DynamicPosition)> {
        let (location, epoch_position, speed) = match *location {
            proto::VehicleLocation::Building { building, interior_pos, speed } => {
                let entity = self.ids.get_building(building)?;
                let building_info = self.building_query.log_get(entity)?;
                let position = building_info.position + interior_pos.xy();
                (Location::Building(entity), position, speed.xy())
            }
            proto::VehicleLocation::Rail { conduit, distance_from_alpha, speed_from_alpha } => {
                let entity = self.ids.get_conduit(conduit)?;
                let &ConduitCorridor(corridor) = self.conduit_query.log_get(entity)?;
                let corridor_info = self.corridor_query.log_get(corridor)?;
                let endpoints = corridor_info.endpoint_positions;
                (
                    Location::Rail(entity),
                    endpoints.alpha + endpoints.atob().normalize_or_zero() * distance_from_alpha,
                    endpoints.atob().normalize_or_zero() * speed_from_alpha,
                )
            }
        };
        Some((location, DynamicPosition { epoch_position, epoch_time: self.time.elapsed(), speed }))
    }
}

#[derive(Component, Reflect)]
pub struct Info {
    pub location:     Location,
    pub ty:           usize,
    pub compartments: Vec<CompartmentInfo>,
}

#[derive(Default, Reflect)]
pub struct CompartmentInfo {
    pub fluid: Option<proto::FluidStorageDetail>,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub enum Location {
    Building(Entity),
    Rail(Entity),
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
    vehicle_query: Query<(&mut Transform, &Info, &DynamicPosition)>,
) {
    for (mut transform, info, dynamic_position) in vehicle_query {
        let pos = dynamic_position.extrapolate(time.elapsed());
        transform.translation = pos.extend(Zorder::Vehicle.z());
    }
}

pub(super) fn on_despawn(_: &mut EntityWorldMut) {
    // reserved
}
