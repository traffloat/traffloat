use std::cmp;

use bevy::app::{self, App, Plugin};
use bevy::asset::{self, AssetServer, Assets};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::name::Name;
use bevy::ecs::query::{With, Without};
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, Query, Res, ResMut, SystemParam};
use bevy::ecs::world::EntityWorldMut;
use bevy::image::Image;
use bevy::math::Vec3;
use bevy::mesh::{Mesh, Mesh2d};
use bevy::picking::Pickable;
use bevy::reflect::Reflect;
use bevy::sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d};
use bevy::transform::components::Transform;
use ordered_float::OrderedFloat;
use traffloat_physics::util::{EntityWorldMutExt, QueryExt, WorldExt};
use traffloat_proto::proto;

use crate::scene::{
    AllHandlersSystemSet, GenericViewable, HandlerClass, IdRegistry, TrackedId, UpdateHandler,
    ViewableKind, Zorder,
};
use crate::util::shapes::Shapes;

mod placement;

pub(super) struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<NeedRearrangeTransform>();
        app.register_type::<BuildingFacilities>();
        app.register_type::<FacilityBuilding>();
        app.register_type::<Info>();
        app.register_type::<TaintOf>();
        app.register_type::<HasTaint>();

        app.add_systems(app::Update, rearrange_facility_tf_system.after(AllHandlersSystemSet));
    }
}

/// List of facilities belonging to a building, component on buildings.
#[derive(Component, Reflect)]
#[relationship_target(relationship = FacilityBuilding, linked_spawn)]
#[require(NeedRearrangeTransform)]
pub struct BuildingFacilities(Vec<Entity>);

/// Marks that a building has had dirty facility changes and transforms need to be rearranged.
#[derive(Component, Default, Reflect)]
pub struct NeedRearrangeTransform(pub bool);

/// Building owning the facility, component on facilities.
#[derive(Component, Reflect)]
#[relationship(relationship_target = BuildingFacilities)]
pub struct FacilityBuilding(pub Entity);

#[derive(Component, Reflect, Default)]
pub struct Info {
    pub volume:       f32,
    pub stored_fluid: Option<proto::FluidStorageFull>,
}

#[derive(Component, Reflect)]
#[relationship(relationship_target = HasTaint)]
struct TaintOf(Entity);

#[derive(Component, Reflect)]
#[relationship_target(relationship = TaintOf, linked_spawn)]
struct HasTaint(Entity);

#[derive(SystemParam)]
pub(super) struct NewFacilityParams<'w, 's> {
    commands:     Commands<'w, 's>,
    ids:          ResMut<'w, IdRegistry>,
    shapes:       Shapes<'w>,
    meshes:       ResMut<'w, Assets<Mesh>>,
    materials:    ResMut<'w, Assets<ColorMaterial>>,
    asset_server: Res<'w, AssetServer>,
}

impl NewFacilityParams<'_, '_> {
    fn load_texture(&self, path: &str) -> asset::Handle<Image> {
        self.asset_server.load(format!("sprites/{path}.png"))
    }
}

impl UpdateHandler for NewFacilityParams<'_, '_> {
    type Update = proto::NewFacility;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Spawn }

    fn handle(&mut self, update: &Self::Update) {
        let texture_handle = self.load_texture(&update.display.sprite_id);

        let Some(&TrackedId::Building(building_entity)) = self.ids.map.get(&update.building) else {
            tracing::error!(
                "Received NewFacility belonging to unknown building id {:?}",
                update.building
            );
            return;
        };

        self.commands.entity(building_entity).queue(|mut entity: EntityWorldMut| {
            entity.insert(NeedRearrangeTransform(true));
        });

        let material = self.materials.add(ColorMaterial {
            texture: Some(texture_handle),
            alpha_mode: AlphaMode2d::Blend,
            ..Default::default()
        });

        let entity = self
            .commands
            .spawn((
                Name::new("Client facility"),
                super::ProtoId(update.id),
                Transform::IDENTITY, // to be reconciled in rearrange_facility_tf_system
                Mesh2d(self.shapes.square()),
                MeshMaterial2d(material),
                Pickable::IGNORE, // to be set to Pickable::default() when building is hovered
                FacilityBuilding(building_entity),
                GenericViewable { name: update.name.clone(), kind: ViewableKind::Facility },
                Info { volume: update.volume, ..Default::default() },
            ))
            .id();

        if let Some(taint) = update.display.taint {
            let texture = self.load_texture(&format!("{}.taint", update.display.sprite_id));
            let taint_material = self.materials.add(ColorMaterial {
                color: taint.into(),
                texture: Some(texture),
                alpha_mode: AlphaMode2d::Blend,
                ..Default::default()
            });
            self.commands.spawn((
                ChildOf(entity),
                TaintOf(entity),
                Mesh2d(self.shapes.square()),
                MeshMaterial2d(taint_material),
                // ensure rendering order is below the facility
                Transform::IDENTITY.with_translation(Vec3::new(
                    0.0,
                    0.0,
                    Zorder::FacilityTaint.z() - Zorder::Facility.z(),
                )),
                // never clickable
                Pickable::IGNORE,
            ));
        }

        self.ids.map.insert(update.id, TrackedId::Facility(entity));
    }
}

#[derive(SystemParam)]
pub struct SetFacilityTaintParams<'w, 's> {
    ids:            ResMut<'w, IdRegistry>,
    facility_query: Query<'w, 's, Option<&'static HasTaint>, With<Info>>,
    taint_query:    Query<'w, 's, &'static MeshMaterial2d<ColorMaterial>>,
    materials:      ResMut<'w, Assets<ColorMaterial>>,
}

impl UpdateHandler for SetFacilityTaintParams<'_, '_> {
    type Update = proto::SetFacilityTaint;

    fn classify(_update: &Self::Update) -> HandlerClass { HandlerClass::Update }

    fn handle(&mut self, update: &Self::Update) {
        let Some(entity) = self.ids.get_facility(update.id) else { return };

        let Some(taint_entity) = self.facility_query.log_get(entity) else { return };
        let Some(taint_entity) = taint_entity else {
            tracing::error!("Facility {entity:?} did not have a taint when created");
            return;
        };
        let Some(material) = self
            .taint_query
            .log_get(taint_entity.0)
            .map(|material| self.materials.get_mut(&material.0).expect("strong material handle"))
        else {
            return;
        };
        material.color = update.taint.into();
    }
}

#[derive(SystemParam)]
pub struct SetFacilityFluidParams<'w, 's> {
    ids:            ResMut<'w, IdRegistry>,
    facility_query: Query<'w, 's, &'static mut Info>,
}

impl UpdateHandler for SetFacilityFluidParams<'_, '_> {
    type Update = proto::SetFacilityFluid;

    fn classify(_update: &Self::Update) -> HandlerClass { HandlerClass::Update }

    fn handle(&mut self, update: &Self::Update) {
        let Some(entity) = self.ids.get_facility(update.id) else { return };
        let Some(mut info) = self.facility_query.log_get_mut(entity) else {
            return;
        };
        info.stored_fluid = Some(update.fluid.clone());
    }
}

fn rearrange_facility_tf_system(
    building_query: Query<
        (&mut NeedRearrangeTransform, &Transform, &BuildingFacilities),
        Without<FacilityBuilding>,
    >,
    mut facility_query: Query<(Entity, &Info, &mut Transform), With<FacilityBuilding>>,
) {
    for (mut need, &building_tf, facilities) in building_query {
        if need.0 {
            let mut facilities: Vec<_> = facilities
                .iter()
                .filter_map(|entity| facility_query.log_get(entity))
                .map(|(entity, info, _)| (entity, info.volume))
                .collect();
            facilities.sort_by_key(|&(_, volume)| cmp::Reverse(OrderedFloat(volume)));
            for (relative_tf, (entity, _)) in placement::compute(facilities.len()).zip(facilities) {
                let (_, _, mut facility_tf) =
                    facility_query.log_get_mut(entity).expect("facility entity should exist");
                *facility_tf = building_tf.mul_transform(relative_tf);
                facility_tf.translation.z = Zorder::Facility.z();
            }
            need.0 = false;
        }
    }
}

pub(super) fn on_despawn(entity: &mut EntityWorldMut) {
    let building = entity.log_get::<FacilityBuilding>().map(|b| b.0);
    if let Some(building) = building {
        entity.world_scope(|world| {
            if let Some(mut marker) = world.log_get_mut::<NeedRearrangeTransform>(building) {
                marker.0 = true;
            }
        });
    }
}
