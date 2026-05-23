use bevy::app::{App, Plugin};
use bevy::ecs::system::Commands;
use bevy::ecs::world::World;
use egui_dock::tab_viewer::OnCloseResponse;

use crate::dock::{self, TabPlacement, new_level};

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, _app: &mut App) {}
}

pub struct Tab;

impl dock::Tab for Tab {
    type TitleSystemParam<'w, 's> = ();
    fn title(&self, param: Self::TitleSystemParam<'_, '_>) -> String { "Main menu".into() }

    type UiSystemParam<'w, 's> = Commands<'w, 's>;
    fn ui(
        &mut self,
        mut commands: Self::UiSystemParam<'_, '_>,
        ui: &mut egui::Ui,
        _: dock::Context,
    ) {
        ui.vertical_centered(|ui| {
            ui.heading("Traffloat");
            if ui.button("New game").clicked() {
                commands.queue(|world: &mut World| {
                    world.resource_mut::<dock::State>().focus_or_create(
                        || new_level::Tab.into(),
                        dock::ReplaceTab(|tab| matches!(tab.tab, dock::TabEnum::NewLevel(_)))
                            .or_always(dock::NewWindow),
                    );
                });
            }
        });
    }

    fn closeable(&self) -> bool { false }

    type OnCloseSystemParam<'w, 's> = ();
    fn on_close(&mut self, (): Self::OnCloseSystemParam<'_, '_>) -> OnCloseResponse {
        OnCloseResponse::Focus
    }

    type BeforeRenderSystemParam<'w, 's> = ();
}
