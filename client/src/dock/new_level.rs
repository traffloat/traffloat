use bevy::app::{App, Plugin};
use bevy::ecs::system::{Commands, ResMut, RunSystemOnce, SystemParam};
use bevy::ecs::world::World;
use egui_dock::tab_viewer::OnCloseResponse;
use traffloat_physics::generate;

use crate::dock::{self, camera};
use crate::scene;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, _app: &mut App) {}
}

pub struct Tab;

#[derive(SystemParam)]
pub struct UiParams<'w, 's> {
    commands: Commands<'w, 's>,
}

impl dock::Tab for Tab {
    type TitleSystemParam<'w, 's> = ();
    fn title(&self, (): Self::TitleSystemParam<'_, '_>) -> String { "New game".into() }

    type UiSystemParam<'w, 's> = UiParams<'w, 's>;
    fn ui(&mut self, mut params: Self::UiSystemParam<'_, '_>, ui: &mut egui::Ui, _: dock::Context) {
        if ui.button("Start").clicked() {
            let config = generate::Config {};
            params.commands.queue(move |world: &mut World| {
                generate::generate(world, config);

                scene::singleplayer::setup(world);

                world
                    .run_system_once(
                        |mut new_camera: camera::NewTabParams, mut dock: ResMut<dock::State>| {
                            let tab = camera::Tab::new(true, "main".into(), &mut new_camera);
                            dock.reset_all(tab.into());
                        },
                    )
                    .unwrap();
            });
        }
    }

    fn closeable(&self) -> bool { false }

    type OnCloseSystemParam<'w, 's> = ();
    fn on_close(&mut self, (): Self::OnCloseSystemParam<'_, '_>) -> OnCloseResponse {
        OnCloseResponse::Focus
    }

    type BeforeRenderSystemParam<'w, 's> = ();
}
