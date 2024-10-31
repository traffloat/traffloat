use std::mem::size_of;
use std::ops::RangeBounds;

use bevy::app::{self, App};
use bevy::asset::{self, AssetServer, Assets};
use bevy::color::{Color, Mix};
use bevy::core_pipeline::core_3d::Camera3d;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::{Changed, QueryData, With, Without};
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, ParamSet, Query, Res, ResMut};
use bevy::gltf::GltfAssetLabel;
use bevy::hierarchy::BuildChildren;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::SpatialBundle;
use bevy::render::mesh::Mesh;
use bevy::state::condition::in_state;
use bevy::transform::components::{GlobalTransform, Transform};
use bevy::{hierarchy, render};
use traffloat_base::{debug, ClientSideSystemSet, UiMutatorSystemSet};
use traffloat_view::appearance::{self, Layer};
use traffloat_view::viewable;

use crate::view::delegate;
use crate::AppState;

pub(super) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            app::Update,
            select_layer_system
                .run_if(in_state(AppState::GameView))
                .in_set(ClientSideSystemSet)
                .in_set(UiMutatorSystemSet),
        );
        app.add_systems(
            app::FixedPreUpdate,
            clone_material_system.run_if(in_state(AppState::GameView)).in_set(ClientSideSystemSet),
        );
        app.add_systems(
            app::PostUpdate,
            propagate_mix_material_system
                .run_if(in_state(AppState::GameView))
                .in_set(ClientSideSystemSet),
        );
    }
}

#[derive(Component)]
pub(super) struct Layered;

#[derive(Component)]
pub(super) struct LayerRefs {
    distal:   Entity,
    proximal: Entity,
    interior: Entity,
}

fn create_mesh_handle(
    assets: &AssetServer,
    glb_ref: appearance::GlbMeshRef,
) -> asset::Handle<Mesh> {
    let mut path_buf = vec![0u8; size_of::<appearance::ResourceSha>() * 2 + 4];
    hex::encode_to_slice(glb_ref.sha.0, &mut path_buf[..glb_ref.sha.0.len() * 2]).unwrap();
    path_buf[glb_ref.sha.0.len() * 2..].copy_from_slice(b".glb");
    let path_str = String::from_utf8(path_buf).unwrap();

    assets.load(
        GltfAssetLabel::Primitive {
            mesh:      glb_ref.mesh as usize,
            primitive: glb_ref.primitive as usize,
        }
        .from_asset(path_str),
    )
}

fn create_material_handle(
    assets: &AssetServer,
    glb_ref: appearance::GlbMaterialRef,
) -> asset::Handle<StandardMaterial> {
    let mut path_buf = vec![0u8; size_of::<appearance::ResourceSha>() * 2 + 4];
    hex::encode_to_slice(glb_ref.sha.0, &mut path_buf[..glb_ref.sha.0.len() * 2]).unwrap();
    path_buf[glb_ref.sha.0.len() * 2..].copy_from_slice(b".glb");
    let path_str = String::from_utf8(path_buf).unwrap();

    assets.load(
        GltfAssetLabel::Material {
            index:             glb_ref.index as usize,
            is_scale_inverted: false,
        }
        .from_asset(path_str),
    )
}

fn spawn_appearance_layer(
    builder: &mut hierarchy::ChildBuilder,
    assets: &AssetServer,
    appearance: &Layer,
    transform: Transform,
    debug_name: &'static str,
) -> Entity {
    let mut layer_entity = builder.spawn((
        SpatialBundle {
            transform,
            visibility: render::view::Visibility::Hidden,
            ..Default::default()
        },
        Layered,
        MixMaterialColor { color: Color::WHITE, factor: 1. },
        PropagatedMixMaterialColor(Color::WHITE),
        debug::Bundle::new(debug_name),
    ));

    if let Layer::Pbr { objects } = appearance {
        layer_entity.with_children(|builder| {
            for &appearance::PbrObject { mesh, material, transform } in objects {
                let material_handle = create_material_handle(assets, material);
                builder.spawn((
                    PbrBundle {
                        mesh: create_mesh_handle(assets, mesh),
                        material: material_handle.clone(),
                        transform: transform.into(),
                        ..Default::default()
                    },
                    CloneMaterialState,
                    BaseMaterialRef { _handle: material_handle },
                    MixMaterialColor { color: Color::WHITE, factor: 1. },
                    PropagatedMixMaterialColor(Color::WHITE),
                    debug::Bundle::new("PbrMesh"),
                ));
            }
        });
    }

    layer_entity.id()
}

pub(super) fn select_layer_system(
    parent_query: Query<
        (
            &render::view::Visibility,
            &render::view::InheritedVisibility,
            &LayerRefs,
            &GlobalTransform,
        ),
        (With<delegate::Marker<viewable::Sid>>, Without<Layered>),
    >,
    mut layer_query: Query<&mut render::view::Visibility, With<Layered>>,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
) {
    let Ok(camera_pos) = camera_query.get_single() else { return };

    parent_query.iter().filter(|(_, vis, _, _)| vis.get()).for_each(
        |(_, _, layers, parent_pos)| {
            let distance_sq = parent_pos.translation().distance_squared(camera_pos.translation());

            // rough estimate display area of the object assuming it is a 1x1x1 cube.
            let transform_det_23 = parent_pos.affine().matrix3.determinant().powf(2. / 3.);
            let far_dist_sq = transform_det_23 * 16.; // TODO make this magic number configurable

            update_layer(&mut layer_query, layers.distal, far_dist_sq.., distance_sq);
            update_layer(
                &mut layer_query,
                layers.proximal,
                transform_det_23..far_dist_sq,
                distance_sq,
            );
            update_layer(&mut layer_query, layers.interior, ..transform_det_23, distance_sq);
        },
    );
}

fn update_layer(
    layer_query: &mut Query<&mut render::view::Visibility, With<Layered>>,
    layer_entity: Entity,
    distance_sq_range: impl RangeBounds<f32>,
    distance_sq: f32,
) {
    let should_display = distance_sq_range.contains(&distance_sq);

    let mut vis =
        layer_query.get_mut(layer_entity).expect("dangling layer entity reference from LayerRefs");
    *vis = if should_display {
        render::view::Visibility::Inherited
    } else {
        render::view::Visibility::Hidden
    };
}

pub(super) fn spawn_all(
    parent: &mut hierarchy::ChildBuilder,
    assets: &AssetServer,
    event: &viewable::ShowMessage,
) -> impl Bundle {
    let distal = spawn_appearance_layer(
        parent,
        assets,
        &event.appearance.distal,
        Transform::IDENTITY,
        "DistalObjectLayer",
    );
    let proximal = spawn_appearance_layer(
        parent,
        assets,
        &event.appearance.proximal,
        Transform::IDENTITY,
        "ProximalObjectLayer",
    );
    let interior = spawn_appearance_layer(
        parent,
        assets,
        &event.appearance.interior,
        Transform::IDENTITY,
        "InteriorObjectLayer",
    );
    LayerRefs { distal, proximal, interior }
}

/// Marker component indicating that the material handle is not cloned yet.
#[derive(Component)]
struct CloneMaterialState;

/// Sets the hierarchical material to use.
#[derive(Component)]
pub struct MixMaterialColor {
    /// The color modifier.
    pub color:  Color,
    /// The factor to mix into the parent color.
    pub factor: f32,
}

#[derive(Component)]
struct PropagatedMixMaterialColor(Color);

/// References the base material definition.
#[derive(Component)]
struct BaseMaterialRef {
    _handle: asset::Handle<StandardMaterial>,
}

fn clone_material_system(
    asset_server: Res<AssetServer>,
    mut assets: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(Entity, &mut asset::Handle<StandardMaterial>), With<CloneMaterialState>>,
    mut commands: Commands,
) {
    for (entity, mut handle) in &mut query {
        match asset_server.load_state(&*handle) {
            asset::LoadState::NotLoaded | asset::LoadState::Loading => continue,
            asset::LoadState::Failed(_) => {
                // abort loading
                commands.entity(entity).remove::<CloneMaterialState>();
            }
            asset::LoadState::Loaded => {
                let material = assets.get(&*handle).expect("asset server reports handle loaded");
                let new_material = material.clone();
                let new_handle = assets.add(new_material);
                *handle = new_handle;

                commands.entity(entity).remove::<CloneMaterialState>();
            }
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
struct PropagationQueryData {
    color_filter:          &'static MixMaterialColor,
    propagated:            &'static mut PropagatedMixMaterialColor,
    material:              Option<&'static asset::Handle<StandardMaterial>>,
    material_clone_marker: Option<&'static CloneMaterialState>,
}

fn propagate_mix_material_system(
    mut materials: ResMut<Assets<StandardMaterial>>,
    changed_object_query: Query<(Entity, Option<&hierarchy::Parent>), Changed<MixMaterialColor>>,
    mut propagation_query: ParamSet<(
        Query<&PropagatedMixMaterialColor>,
        Query<PropagationQueryData>,
    )>,
    children_query: Query<&hierarchy::Children, With<PropagatedMixMaterialColor>>,
) {
    fn propagate(
        materials: &mut Assets<StandardMaterial>,
        propagation_query: &mut Query<PropagationQueryData>,
        children_query: &Query<&hierarchy::Children, With<PropagatedMixMaterialColor>>,
        entity: Entity,
        parent_color: Color,
    ) {
        let Ok(mut data) = propagation_query.get_mut(entity) else {
            return; // not a child object for rendering
        };

        let mixed = parent_color.mix(&data.color_filter.color, data.color_filter.factor);
        data.propagated.0 = mixed;

        if let (None, Some(material_handle)) = (data.material_clone_marker, data.material) {
            match materials.get_mut(material_handle) {
                None => bevy::log::warn!("cloned material handle does not have material in store"),
                Some(material) => {
                    material.base_color = mixed;
                }
            };
        }

        if let Ok(children) = children_query.get(entity) {
            for &child in children {
                propagate(materials, propagation_query, children_query, child, mixed);
            }
        }
    }

    changed_object_query.iter().for_each(|(entity, parent)| {
        let parent_color = if let Some(parent) = parent {
            if let Ok(propagated) = propagation_query.p0().get(parent.get()) {
                propagated.0
            } else {
                Color::WHITE
            }
        } else {
            Color::WHITE
        };

        propagate(
            &mut materials,
            &mut propagation_query.p1(),
            &children_query,
            entity,
            parent_color,
        );
    });
}
