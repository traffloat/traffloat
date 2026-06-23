use bevy::app::{App, Plugin};
use bevy::ecs::system::Commands;
use egui_dock::tab_viewer::OnCloseResponse;

use crate::dock::{self, DockCommand, TabPlacement, new_level, save};

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
                commands.queue(DockCommand(|dock| {
                    dock.focus_or_create(
                        || new_level::Tab.into(),
                        dock::ReplaceTab(|state| state.tab.is_new_level())
                            .or_always(dock::NewWindow),
                    );
                }));
            }
            if ui.button("Load game").clicked() {
                commands.queue(DockCommand(|dock| {
                    dock.focus_or_create(
                        || save::OpenTab::default().into(),
                        dock::ReplaceTab(|state| state.tab.is_open_save())
                            .or_always(dock::NewWindow),
                    );
                }));
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
