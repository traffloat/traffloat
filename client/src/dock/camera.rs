use bevy::app::{App, Plugin};
use bevy::asset::{self, Assets};
use bevy::camera::{Camera, Camera2d, ClearColorConfig, ImageRenderTarget, RenderTarget, Viewport};
use bevy::color::Color;
use bevy::ecs::entity::Entity;
use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{Commands, Query, ResMut, SystemParam};
use bevy::image::Image;
use bevy::math::{UVec2, Vec2};
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::transform::components::{GlobalTransform, Transform};
use bevy_egui::helpers::egui_vec2_into_vec2;
use bevy_egui::{EguiPrimaryContextPass, EguiTextureHandle, EguiUserTextures};
use bevy_mod_config::{AppExt, Config};
use egui::load::SizedTexture;
use traffloat_physics::util::QueryExt;

use crate::{ConfigManager, dock};

mod input;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>();
        app.init_config::<ConfigManager, Conf>("camera");
        app.add_systems(EguiPrimaryContextPass, UiState::cleanup.before(super::render_system));
    }
}

pub struct Tab {
    pub is_main:  bool,
    title:        String,
    /// The bevy camera entity.
    camera:       Entity,
    image_handle: asset::Handle<Image>,
    image_id:     Option<egui::TextureId>,
}

#[derive(SystemParam)]
pub struct NewTabParams<'w, 's> {
    images:   ResMut<'w, Assets<Image>>,
    textures: ResMut<'w, EguiUserTextures>,
    commands: Commands<'w, 's>,
}

impl Tab {
    pub fn new(is_main: bool, title: String, params: &mut NewTabParams) -> Self {
        let image = Image::new_target_texture(512, 512, TextureFormat::Bgra8UnormSrgb, None);
        let image_handle = params.images.add(image);
        let image_id = params.textures.add_image(EguiTextureHandle::Strong(image_handle.clone()));
        let camera = params
            .commands
            .spawn((
                Camera2d,
                RenderTarget::Image(ImageRenderTarget {
                    handle:       image_handle.clone(),
                    scale_factor: 1.0,
                }),
                Camera { clear_color: ClearColorConfig::Default, ..Default::default() },
            ))
            .id();
        Tab { is_main, title, camera, image_handle, image_id: Some(image_id) }
    }
}

impl dock::Tab for Tab {
    type TitleSystemParam<'w, 's> = ();

    fn title(&self, (): Self::TitleSystemParam<'_, '_>) -> String {
        format!("Viewport: {}", &self.title)
    }

    type UiSystemParam<'w, 's> = UiSystemParam<'w, 's>;

    fn ui(
        &mut self,
        mut param: Self::UiSystemParam<'_, '_>,
        ui: &mut egui::Ui,
        dock: dock::Context,
    ) {
        let Some((mut camera, mut camera_tf, global_tf)) =
            param.camera_query.log_get_mut(self.camera)
        else {
            return;
        };
        let viewport_size = ui.max_rect().size();
        #[expect(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            reason = "viewport dimensions should be within bounds"
        )]
        let physical_size = UVec2 { x: viewport_size.x as u32, y: viewport_size.y as u32 };
        camera.viewport = Some(Viewport {
            physical_position: UVec2 { x: 0, y: 0 },
            physical_size,
            depth: 0.0..1.0,
        });
        camera.order = -isize::try_from(dock.order).expect("tab order is within isize bounds") - 1;

        let image = param.images.get_mut(&self.image_handle).expect("strong handle");
        #[expect(clippy::cast_sign_loss, reason = "rect dimensions should be nonnegative")]
        #[expect(
            clippy::cast_possible_truncation,
            reason = "rect dimensions should be within u32 bounds"
        )]
        image.resize(Extent3d {
            width: viewport_size.x as u32,
            height: viewport_size.y as u32,
            ..Default::default()
        });

        let Some(image_id) = self.image_id else { return };
        let resp = ui.add(
            egui::Image::new(SizedTexture::new(image_id, viewport_size))
                .sense(egui::Sense::click_and_drag()),
        );

        if dock.focused {
            param.input.consume_input(&mut camera_tf, &resp);
        }

        if let Some(hover_pos) = resp.hover_pos() {
            let viewport_pos = hover_pos - resp.rect.min;
            match camera.viewport_to_world_2d(global_tf, egui_vec2_into_vec2(viewport_pos)) {
                Ok(world_pos) => {
                    param.ui_state.hover_state = Some(HoverState {
                        camera: self.camera,
                        viewport_pos,
                        world_pos,
                        primary_clicked: resp.clicked(),
                        secondary_clicked: resp.secondary_clicked(),
                    });
                }
                Err(err) => bevy::log::warn_once!(
                    "viewport position {viewport_pos:} should be in world: {err}"
                ),
            }
        }
    }

    fn closeable(&self) -> bool { !self.is_main }
    type OnCloseSystemParam<'w, 's> = ();

    type BeforeRenderSystemParam<'w, 's> = ();
    fn before_render(
        &mut self,
        contexts: &mut bevy_egui::EguiContexts,
        _param: Self::BeforeRenderSystemParam<'_, '_>,
    ) {
        // self.image_id = contexts.image_id(&self.image_handle);
    }
}

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    camera_query:
        Query<'w, 's, (&'static mut Camera, &'static mut Transform, &'static GlobalTransform)>,
    images:       ResMut<'w, Assets<Image>>,
    ui_state:     ResMut<'w, UiState>,
    input:        input::Param<'w, 's>,
}

#[derive(Resource, Default)]
pub struct UiState {
    pub hover_state: Option<HoverState>,
}

impl UiState {
    fn cleanup(mut this: ResMut<UiState>) { this.hover_state = None; }
}

pub struct HoverState {
    pub camera:            Entity,
    pub viewport_pos:      egui::Vec2,
    pub world_pos:         Vec2,
    pub primary_clicked:   bool,
    pub secondary_clicked: bool,
}

#[derive(Config)]
pub struct Conf {
    #[config(default = 320.0)]
    pub movement_speed: f32,
    #[config(default = 1.0)]
    pub rotation_speed: f32,
    #[config(default = 1.4)]
    pub zoom_rate:      f32,
}
