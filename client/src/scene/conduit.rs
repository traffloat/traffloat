use std::collections::HashMap;
use std::f32::consts::{FRAC_1_SQRT_2, SQRT_2};
use std::{cmp, mem};

use bevy::app::{self, App, Plugin};
use bevy::asset::{self, AssetServer, Assets};
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::message::{Message, MessageReader};
use bevy::ecs::name::Name;
use bevy::ecs::observer;
use bevy::ecs::query::{Has, With, Without};
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, ParamSet, Query, Res, ResMut, Single, SystemParam};
use bevy::ecs::world::{EntityWorldMut, World};
use bevy::image::Image;
use bevy::math::primitives::Annulus;
use bevy::math::{Quat, Vec2, Vec3};
use bevy::mesh::{Mesh, Mesh2d};
use bevy::picking::{Pickable, events as pick};
use bevy::reflect::Reflect;
use bevy::sprite_render::{AlphaMode2d, ColorMaterial, MeshMaterial2d};
use bevy::transform::components::Transform;
use bevy_mesh::VertexAttributeValues;
use bevy_mod_config::{AppExt, Config, ReadConfig};
use either::Either;
use ordered_float::OrderedFloat;
use traffloat_physics::util::{EntityWorldMutExt, QueryExt, WorldExt};
use traffloat_physics::{try_log, view};
use traffloat_proto::proto;

use crate::ConfigManager;
use crate::scene::picking::{self, ObservePicking};
use crate::scene::{
    AllHandlersSystemSet, GenericViewable, HandlerClass, IdRegistry, TrackedId, UpdateHandler,
    ViewableKind, Zorder, corridor,
};
use crate::util::shapes::Shapes;

pub(super) struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.register_type::<NeedRearrangeTransform>();
        app.register_type::<CorridorConduits>();
        app.register_type::<ConduitCorridor>();
        app.register_type::<Info>();
        app.register_type::<HasConduitOutline>();
        app.register_type::<ConduitOutlineOf>();

        app.add_systems(app::Update, rearrange_conduit_tf_system.after(AllHandlersSystemSet));
    }
}

#[derive(Component, Reflect)]
#[relationship_target(relationship = ConduitCorridor)]
#[require(NeedRearrangeTransform)]
pub struct CorridorConduits(Vec<Entity>);

#[derive(Component, Reflect)]
#[relationship(relationship_target = CorridorConduits)]
pub struct ConduitCorridor(pub Entity);

#[derive(Component, Reflect, Default)]
struct NeedRearrangeTransform(bool);

#[derive(Component, Reflect)]
pub struct Info {
    pub ty:           proto::ConduitType,
    pub radius:       f32,
    pub stored_fluid: Option<proto::FluidStorageFull>,
}

#[derive(SystemParam)]
pub(super) struct NewConduitParams<'w, 's> {
    pub commands:  Commands<'w, 's>,
    pub ids:       ResMut<'w, IdRegistry>,
    pub shapes:    Shapes<'w>,
    pub materials: ResMut<'w, Assets<ColorMaterial>>,
}

impl UpdateHandler for NewConduitParams<'_, '_> {
    type Update = proto::NewConduit;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Spawn }

    fn handle(&mut self, update: &Self::Update) {
        let Some(&TrackedId::Corridor(corridor_entity)) = self.ids.map.get(&update.corridor) else {
            tracing::error!(
                "Received NewConduit belonging to unknown corridor id {:?}",
                update.corridor
            );
            return;
        };

        self.commands.entity(corridor_entity).queue(|mut entity: EntityWorldMut| {
            entity.insert(NeedRearrangeTransform(true));
        });

        let material = self.materials.add(match update.ty {
            proto::ConduitType::FluidPipe => ColorMaterial::from_color(Color::NONE),
        });

        let entity = self
            .commands
            .spawn((
                Name::new("Client conduit"),
                super::ProtoId(update.id),
                Transform::IDENTITY, // to be reconciled in rearrange_conduit_tf_system
                Mesh2d(self.shapes.square()),
                MeshMaterial2d(material),
                Pickable::IGNORE, // to be set to Pickable::default() when corridor is hovered
                ConduitCorridor(corridor_entity),
                GenericViewable { name: update.name.clone(), kind: ViewableKind::Conduit },
                Info { ty: update.ty, radius: update.radius, stored_fluid: None },
            ))
            .id();

        self.ids.map.insert(update.id, TrackedId::Conduit(entity));
    }
}

#[derive(SystemParam)]
pub(super) struct UpdateFluidConduitParams<'w, 's> {
    ids:           Res<'w, IdRegistry>,
    materials:     ResMut<'w, Assets<ColorMaterial>>,
    conduit_query: Query<'w, 's, (&'static MeshMaterial2d<ColorMaterial>, &'static mut Info)>,
}

impl UpdateHandler for UpdateFluidConduitParams<'_, '_> {
    type Update = proto::UpdateFluidConduit;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Update }

    fn handle(&mut self, update: &Self::Update) {
        let Some(conduit_entity) = self.ids.get_conduit(update.id) else { return };

        let Ok((handle, mut info)) = self.conduit_query.get_mut(conduit_entity) else {
            return;
        };
        let material = try_log!(self.materials.get_mut(&handle.0), expect "corridor entity should reference a valid material" or return);
        material.color = update.color.into();

        info.stored_fluid = None;
    }
}

#[derive(SystemParam)]
pub(super) struct UpdateFluidConduitFullParams<'w, 's> {
    ids:           Res<'w, IdRegistry>,
    materials:     ResMut<'w, Assets<ColorMaterial>>,
    conduit_query: Query<'w, 's, (&'static MeshMaterial2d<ColorMaterial>, &'static mut Info)>,
}

impl UpdateHandler for UpdateFluidConduitFullParams<'_, '_> {
    type Update = proto::UpdateFluidConduitFull;

    fn classify(update: &Self::Update) -> HandlerClass { HandlerClass::Update }

    fn handle(&mut self, update: &Self::Update) {
        let Some(conduit_entity) = self.ids.get_conduit(update.id) else { return };

        let Ok((handle, mut info)) = self.conduit_query.get_mut(conduit_entity) else { return };
        let material = try_log!(self.materials.get_mut(&handle.0), expect "corridor entity should reference a valid material" or return);
        material.color = update.color.into();

        info.stored_fluid = Some(update.fluid.clone());
    }
}

/// References the conduit outlines *from the corridor*.
#[derive(Component, Reflect)]
#[relationship_target(relationship = ConduitOutlineOf)]
pub struct HasConduitOutline(Entity);

/// References the *corridor* from the conduit outline entity.
#[derive(Component, Reflect)]
#[relationship(relationship_target = HasConduitOutline)]
pub struct ConduitOutlineOf(pub Entity);

fn rearrange_conduit_tf_system(
    corridor_query: Query<
        (
            &mut NeedRearrangeTransform,
            &Transform,
            &CorridorConduits,
            &corridor::Info,
            &HasConduitOutline,
        ),
        Without<ConduitCorridor>,
    >,
    mut conduit_query: Query<(Entity, &Info, &mut Transform), With<ConduitCorridor>>,
    outline_query: Query<&Mesh2d, With<ConduitOutlineOf>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (mut need, &corridor_tf, conduits, corridor_info, outline) in corridor_query {
        if need.0 {
            let Some(outline_mesh) = outline_query.log_get(outline.0) else { continue };
            let outline_mesh = meshes
                .get_mut(&outline_mesh.0)
                .expect("getting asset by strong handle should succeed");
            let Some(VertexAttributeValues::Float32x3(mesh_positions)) =
                outline_mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
            else {
                panic!("Position attribute was initialized as Float32x3 during spawn");
            };
            mesh_positions.clear();

            let mut conduits: Vec<_> = conduits
                .iter()
                .filter_map(|entity| conduit_query.log_get(entity))
                .map(|(entity, info, _)| (entity, info.radius))
                .collect();
            conduits.sort_by_key(|&(_, radius)| OrderedFloat(radius));

            let mut last_y2 = None;
            for ((relative_tf, y1, y2), &(entity, _)) in
                compute_placement(conduits.iter().map(|&(_, radius)| radius), corridor_info.radius)
                    .zip(&conduits)
            {
                let (_, _, mut conduit_tf) =
                    conduit_query.log_get_mut(entity).expect("conduit entity should exist");
                *conduit_tf = corridor_tf.mul_transform(relative_tf);
                conduit_tf.translation.z = Zorder::Conduit.z();

                mesh_positions.push([0.5, y1, 0.0]);
                mesh_positions.push([-0.5, y1, 0.0]);
                dbg!(y1, y2);
                last_y2 = Some(y2);
            }
            if let Some(last_y2) = last_y2 {
                // Add a final quad to fill the remaining space in the corridor outline
                mesh_positions.push([0.5, last_y2, 0.0]);
                mesh_positions.push([-0.5, last_y2, 0.0]);
            }

            need.0 = false;
        }
    }
}

/// `radii` is an increasing sequence.
fn compute_placement(
    radii: impl ExactSizeIterator<Item = f32> + Clone,
    corridor_radius: f32,
) -> impl Iterator<Item = (Transform, f32, f32)> {
    let num_conduits = radii.len();
    (num_conduits != 0)
        .then(|| {
            let total = corridor_radius.powi(2);

            let radius_sum: f32 = radii.clone().into_iter().sum();
            let radius_ratio = (radius_sum / corridor_radius).clamp(0.3, 0.8);
            let width_scale = radius_ratio / (radius_sum / corridor_radius);
            let mut next_y_offset = -radius_ratio * 0.5;

            dbg!(width_scale);
            radii.into_iter().map(move |radius| {
                let scale = (radius / corridor_radius) * width_scale;

                let y_offset = next_y_offset;
                next_y_offset += scale;

                (
                    Transform {
                        translation: Vec3::new(
                            0.0,
                            y_offset,
                            Zorder::Conduit.z() - Zorder::Corridor.z(),
                        ),
                        rotation:    Quat::IDENTITY,
                        scale:       Vec3::new(1.0, scale, 1.0),
                    },
                    y_offset,
                    next_y_offset,
                )
            })
        })
        .into_iter()
        .flatten()
}

pub(super) fn on_despawn(entity: &mut EntityWorldMut) {
    let corridor = entity.log_get::<ConduitCorridor>().map(|b| b.0);
    if let Some(corridor) = corridor {
        entity.world_scope(|world| {
            if let Some(mut marker) = world.log_get_mut::<NeedRearrangeTransform>(corridor) {
                marker.0 = true;
            }
        });
    }
}
