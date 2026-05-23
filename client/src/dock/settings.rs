use bevy::app::{App, Plugin};
use bevy::ecs::system::Commands;
use bevy::ecs::world::World;
use egui_dock::tab_viewer::OnCloseResponse;

use crate::dock::{self, TabPlacement, new_level};

pub struct Tab;

impl dock::Tab for Tab {
    type TitleSystemParam<'w, 's> = ();
    fn title(&self, param: Self::TitleSystemParam<'_, '_>) -> String { "Settings".into() }

    type UiSystemParam<'w, 's> = bevy_mod_config::manager::egui::Display<'w, 's>;
    fn ui(&mut self, mut param: Self::UiSystemParam<'_, '_>, ui: &mut egui::Ui, _: dock::Context) {
        param.show(ui);
    }

    type OnCloseSystemParam<'w, 's> = ();

    type BeforeRenderSystemParam<'w, 's> = ();
}
