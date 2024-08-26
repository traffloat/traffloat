use std::f32::consts::{FRAC_PI_2, TAU};
use std::{iter, mem};

use bevy::app::{self, App};
use bevy::asset::{AssetId, AssetServer, Handle};
use bevy::core_pipeline::core_3d::{Opaque3d, Opaque3dBinKey};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::{ROQueryItem, With};
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::{Commands, Query, Res, ResMut, Resource, SystemParamItem};
use bevy::ecs::world::{FromWorld, World};
use bevy::math::{Mat3A, Vec3, Vec3A};
use bevy::render;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_phase::{
    AddRenderCommand, BinnedRenderPhaseType, DrawFunctions, PhaseItem, RenderCommand,
    RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewBinnedRenderPhases,
};
use bevy::render::render_resource::{
    BufferUsages, ColorTargetState, ColorWrites, FragmentState, IndexFormat, MultisampleState,
    PipelineCache, PrimitiveState, RawBufferVec, RenderPipelineDescriptor, Shader,
    SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};
use bevy::render::texture::BevyDefault;
use bevy::render::view::ExtractedView;
use bevy::render::{Render, RenderApp, RenderSet};
use bevy::state::state;
use bytemuck::{Pod, Zeroable};
use rand::{self, distributions, Rng};
use rand_distr::LogNormal;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256StarStar;

use crate::AppState;

pub(crate) struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(state::OnEnter(AppState::GameView), setup)
            .add_plugins(ExtractComponentPlugin::<StarList>::default());

        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .init_resource::<StarPipeline>()
            .init_resource::<SpecializedRenderPipelines<StarPipeline>>()
            .add_render_command::<Opaque3d, (SetItemPipeline, DrawStars)>()
            .add_systems(
                Render,
                (
                    prepare_star_buffers_system.in_set(RenderSet::Prepare),
                    queue_star_system.in_set(RenderSet::Queue),
                ),
            );
    }
}

fn setup(mut commands: Commands) {
    // TODO config should be based on saved value instead.
    let seed = [0; 32];
    let num_stars = 1000;

    let mut rng = Xoshiro256StarStar::from_seed(seed);

    let mut triangles = vec![[Vec3::ZERO; 4]; num_stars];
    let mut colors = vec![Vec3::ZERO; num_stars];

    for offset in 0..num_stars {
        let color = [(); 3].map(|()| rng.sample(distributions::Uniform::new_inclusive(0.6, 1.)));
        colors[offset] = Vec3::from_array(color);

        let center = Vec3A::X;
        let right_dir = Vec3A::Y * rng.sample(LogNormal::<f32>::new(0., 0.5).unwrap());
        let down_dir = Vec3A::Z * rng.sample(LogNormal::<f32>::new(0., 0.5).unwrap());

        let transform =
            Mat3A::from_rotation_z(
                rng.sample(distributions::Uniform::new_inclusive(-FRAC_PI_2, FRAC_PI_2)),
            ) * Mat3A::from_rotation_y(rng.sample(distributions::Uniform::new(0., TAU)))
                * Mat3A::from_rotation_x(rng.sample(distributions::Uniform::new(0., FRAC_PI_2)));

        let up = transform * (center - down_dir);
        let down = transform * (center + down_dir);
        let left = transform * (center - right_dir);
        let right = transform * (center + right_dir);

        triangles[offset] = [up, right, down, left].map(Vec3::from);
    }

    commands.spawn((super::Owned, StarList { quads: triangles, colors }));
}

fn prepare_star_buffers_system(mut commands: Commands) { commands.init_resource::<StarBuffers>(); }

/// Specifies the stars to render.
#[derive(Clone, Component, ExtractComponent)]
pub struct StarList {
    pub quads:  Vec<[Vec3; 4]>,
    pub colors: Vec<Vec3>,
}

#[derive(Resource)]
struct StarBuffers {
    vertices: RawBufferVec<Vertex>,
    indices:  RawBufferVec<u16>,
}

impl FromWorld for StarBuffers {
    fn from_world(world: &mut World) -> Self {
        let mut query = world.query::<&StarList>();
        let mut lists = query.iter_mut(world);

        let mut vertices = RawBufferVec::new(BufferUsages::VERTEX);
        let mut indices = RawBufferVec::new(BufferUsages::INDEX);

        if let Some(list) = lists.next() {
            vertices.extend(
                iter::zip(&list.quads, &list.colors)
                    .flat_map(|(&quad, &color)| quad.map(move |pos| Vertex { pos, color })),
            );
            indices.extend((0..list.quads.len()).flat_map(|quad| {
                [quad * 4, quad * 4 + 1, quad * 4 + 3, quad * 4 + 2, quad * 4 + 3, quad * 4 + 1]
                    .map(|index| u16::try_from(index).expect("too many stars"))
            }));
        }

        Self { vertices, indices }
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    pos:   Vec3,
    color: Vec3,
}

struct DrawStars;

impl<P: PhaseItem> RenderCommand<P> for DrawStars {
    type Param = SRes<StarBuffers>;
    type ViewQuery = ();
    type ItemQuery = ();

    fn render<'w>(
        _: &P,
        (): ROQueryItem<'w, ()>,
        _: Option<ROQueryItem<'w, ()>>,
        buffers: SystemParamItem<'w, '_, SRes<StarBuffers>>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let buffers = buffers.into_inner();
        pass.set_vertex_buffer(0, buffers.vertices.buffer().unwrap().slice(..));
        pass.set_index_buffer(buffers.indices.buffer().unwrap().slice(..), 0, IndexFormat::Uint16);
        pass.draw_indexed(0..buffers.indices.len() as u32, 0, 0..1);

        RenderCommandResult::Success
    }
}

fn queue_star_system(
    pipeline_cache: Res<PipelineCache>,
    star_pipeline: Res<StarPipeline>,
    opaque_draw_functions: Res<DrawFunctions<Opaque3d>>,
    views: Query<Entity, With<ExtractedView>>,
    star_list_query: Query<Entity>,
    mut opaque_render_phases: ResMut<ViewBinnedRenderPhases<Opaque3d>>,
    mut specialized_render_pipelines: ResMut<SpecializedRenderPipelines<StarPipeline>>,
    msaa: Res<render::view::Msaa>,
) {
    let draw_stars = opaque_draw_functions.read().id::<(SetItemPipeline, DrawStars)>();

    for view_entity in views.iter() {
        let Some(phase) = opaque_render_phases.get_mut(&view_entity) else {
            continue;
        };

        for star_list_entity in &star_list_query {
            let pipeline_id =
                specialized_render_pipelines.specialize(&pipeline_cache, &star_pipeline, *msaa);
            phase.add(
                Opaque3dBinKey {
                    draw_function:          draw_stars,
                    pipeline:               pipeline_id,
                    asset_id:               AssetId::<Shader>::invalid().untyped(),
                    material_bind_group_id: None,
                    lightmap_image:         None,
                },
                star_list_entity,
                BinnedRenderPhaseType::NonMesh,
            );
        }
    }
}

#[derive(Resource)]
struct StarPipeline {
    shader: Handle<Shader>,
}

impl FromWorld for StarPipeline {
    fn from_world(world: &mut World) -> Self {
        // Load and compile the shader in the background.
        let asset_server = world.resource::<AssetServer>();

        StarPipeline { shader: asset_server.load("shaders/stars.wgsl") }
    }
}

impl SpecializedRenderPipeline for StarPipeline {
    type Key = render::view::Msaa;

    fn specialize(&self, msaa: render::view::Msaa) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label:                Some("star rendering".into()),
            layout:               Vec::new(),
            push_constant_ranges: Vec::new(),
            vertex:               VertexState {
                shader:      self.shader.clone(),
                shader_defs: Vec::new(),
                entry_point: "vertex".into(),
                buffers:     vec![VertexBufferLayout {
                    array_stride: mem::size_of::<Vertex>() as u64,
                    step_mode:    VertexStepMode::Vertex,
                    attributes:   vec![
                        VertexAttribute {
                            format:          VertexFormat::Float32x3,
                            offset:          0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format:          VertexFormat::Float32x3,
                            offset:          std::mem::offset_of!(Vertex, color) as u64,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            fragment:             Some(FragmentState {
                shader:      self.shader.clone(),
                shader_defs: Vec::new(),
                entry_point: "fragment".into(),
                targets:     vec![Some(ColorTargetState {
                    format:     TextureFormat::bevy_default(),
                    blend:      None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive:            PrimitiveState::default(),
            depth_stencil:        None,
            multisample:          MultisampleState {
                count:                     msaa.samples(),
                mask:                      !0,
                alpha_to_coverage_enabled: false,
            },
        }
    }
}
