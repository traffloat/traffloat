use std::mem::size_of;
use std::ops::RangeBounds;

use bevy::app::{self, App};
use bevy::asset::{AssetServer, Handle};
use bevy::core_pipeline::core_3d::Camera3d;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::{With, Without};
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::Query;
use bevy::gltf::GltfAssetLabel;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::SpatialBundle;
use bevy::render::mesh::Mesh;
use bevy::state::condition::in_state;
use bevy::transform::components::{GlobalTransform, Transform};
use bevy::{hierarchy, render};
use traffloat_base::debug;
use traffloat_view::appearance::{self, Layer};
use traffloat_view::viewable;

use crate::view::delegate;
use crate::AppState;

pub(super) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(app::Update, select_layer_system.run_if(in_state(AppState::GameView)));
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

fn create_mesh_handle(assets: &AssetServer, glb_ref: appearance::GlbMeshRef) -> Handle<Mesh> {
    let mut path_buf = vec![0u8; size_of::<appearance::GlbSha>() * 2 + 4];
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
) -> Handle<StandardMaterial> {
    let mut path_buf = vec![0u8; size_of::<appearance::GlbSha>() * 2 + 4];
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
    appearance: Layer,
    transform: Transform,
    debug_name: &'static str,
) -> Entity {
    match appearance {
        Layer::Null => builder
            .spawn((
                SpatialBundle {
                    transform,
                    visibility: render::view::Visibility::Hidden,
                    ..Default::default()
                },
                Layered,
                debug::Bundle::new(debug_name),
            ))
            .id(),
        Layer::Pbr { mesh, material } => builder
            .spawn((
                PbrBundle {
                    mesh: create_mesh_handle(assets, mesh),
                    material: create_material_handle(assets, material),
                    transform,
                    visibility: render::view::Visibility::Hidden,
                    ..Default::default()
                },
                Layered,
                debug::Bundle::new(debug_name),
            ))
            .id(),
    }
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
    event: &viewable::ShowEvent,
) -> impl Bundle {
    let distal = spawn_appearance_layer(
        parent,
        assets,
        event.appearance.distal,
        Transform::IDENTITY,
        "DistalObjectLayer",
    );
    let proximal = spawn_appearance_layer(
        parent,
        assets,
        event.appearance.proximal,
        Transform::IDENTITY,
        "ProximalObjectLayer",
    );
    let interior = spawn_appearance_layer(
        parent,
        assets,
        event.appearance.interior,
        Transform::IDENTITY,
        "InteriorObjectLayer",
    );
    LayerRefs { distal, proximal, interior }
}
