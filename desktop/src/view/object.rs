use std::mem::size_of;
use std::ops::RangeBounds;

use bevy::app::{self, App};
use bevy::asset::{AssetServer, Handle};
use bevy::core_pipeline::core_3d::Camera3d;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::EventReader;
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Query, Res, ResMut, Resource};
use bevy::gltf::GltfAssetLabel;
use bevy::hierarchy::BuildChildren;
use bevy::pbr::{PbrBundle, StandardMaterial};
use bevy::prelude::SpatialBundle;
use bevy::render;
use bevy::render::mesh::Mesh;
use bevy::state::condition::in_state;
use bevy::transform::components::{GlobalTransform, Transform};
use bevy::utils::HashMap;
use traffloat_base::EventReaderSystemSet;
use traffloat_view::appearance::{self, Appearance};
use traffloat_view::viewable;

use crate::AppState;

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DelegateSidIndex>();

        app.add_systems(app::Update, select_layer_system.run_if(in_state(AppState::GameView)));
        app.add_systems(
            app::Update,
            handle_show_system.in_set(EventReaderSystemSet::<viewable::ShowEvent>::default()),
        );
    }
}

/// Marks the entity as the delegate visualization for the SID.
#[derive(Debug, Clone, Copy, Component, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DelegateViewable(viewable::Sid);

// We cannot reuse the main SidIndex because it will overlap the entities
// if simulation is in the same world.
#[derive(Default, Resource)]
struct DelegateSidIndex(HashMap<viewable::Sid, Entity>);

#[derive(Component)]
struct LayerRefs {
    distal:   Entity,
    proximal: Entity,
    interior: Entity,
}

fn create_mesh_handle(assets: &AssetServer, glb_ref: appearance::GlbMeshRef) -> Handle<Mesh> {
    let mut path_buf = vec![0u8; size_of::<appearance::GlbSha>() * 2 + 4];
    hex::encode_to_slice(glb_ref.sha, &mut path_buf[..glb_ref.sha.len() * 2]).unwrap();
    path_buf[glb_ref.sha.len() * 2..].copy_from_slice(b".glb");
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
    hex::encode_to_slice(glb_ref.sha, &mut path_buf[..glb_ref.sha.len() * 2]).unwrap();
    path_buf[glb_ref.sha.len() * 2..].copy_from_slice(b".glb");
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
    commands: &mut Commands,
    assets: &AssetServer,
    appearance: Appearance,
    transform: Transform,
) -> Entity {
    match appearance {
        Appearance::Null => commands.spawn_empty().id(),
        Appearance::Pbr { mesh, material } => commands
            .spawn((
                PbrBundle {
                    mesh: create_mesh_handle(assets, mesh),
                    material: create_material_handle(assets, material),
                    transform,
                    visibility: render::view::Visibility::Visible,
                    ..Default::default()
                },
                Layered,
            ))
            .id(),
    }
}

#[derive(Component)]
struct Layered;

fn handle_show_system(
    mut commands: Commands,
    mut reader: EventReader<viewable::ShowEvent>,
    mut sid_index: ResMut<DelegateSidIndex>,
    assets: Res<AssetServer>,
) {
    for ev in reader.read() {
        let viewable_id = *sid_index.0.entry(ev.viewable).or_insert_with(|| {
            let distal = spawn_appearance_layer(
                &mut commands,
                &assets,
                ev.appearance.distal,
                Transform::IDENTITY,
            );
            let proximal = spawn_appearance_layer(
                &mut commands,
                &assets,
                ev.appearance.proximal,
                Transform::IDENTITY,
            );
            let interior = spawn_appearance_layer(
                &mut commands,
                &assets,
                ev.appearance.interior,
                Transform::IDENTITY,
            );
            let layer_refs = LayerRefs { distal, proximal, interior };

            let mut parent = commands.spawn((
                DelegateViewable(ev.viewable),
                ev.appearance,
                layer_refs,
                SpatialBundle {
                    visibility: render::view::Visibility::Hidden,
                    transform: Transform::from(ev.transform),
                    ..<_>::default()
                },
            ));
            parent.push_children(&[distal, proximal, interior]);

            parent.id()
        });

        commands.entity(viewable_id).insert(render::view::Visibility::Visible);
    }
}

fn select_layer_system(
    parent_query: Query<
        (&render::view::ViewVisibility, &LayerRefs, &GlobalTransform),
        With<DelegateViewable>,
    >,
    mut layer_query: Query<&mut render::view::Visibility>,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
) {
    let Ok(camera_pos) = camera_query.get_single() else { return };

    parent_query.iter().filter(|(vis, _, _)| vis.get()).for_each(|(_, layers, parent_pos)| {
        let distance_sq = parent_pos.translation().distance_squared(camera_pos.translation());

        // rough estimate display area of the object assuming it is a 1x1x1 cube.
        let transform_det_23 = parent_pos.affine().matrix3.determinant().powf(2. / 3.);
        let far_dist_sq = transform_det_23 * 16.; // TODO make this magic number configurable

        update_layer(&mut layer_query, layers.distal, far_dist_sq.., distance_sq);
        update_layer(&mut layer_query, layers.proximal, transform_det_23..far_dist_sq, distance_sq);
        update_layer(&mut layer_query, layers.interior, ..transform_det_23, distance_sq);
    });
}

fn update_layer(
    layer_query: &mut Query<&mut render::view::Visibility>,
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
