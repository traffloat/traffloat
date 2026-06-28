use bevy::ecs::system::Commands;
use egui_material_icons::{MaterialIcon, icons};

use crate::dock::{self, DockCommand, TabPlacement, menu};

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

#[derive(Default)]
pub struct MenuAction;

impl menu::Action for MenuAction {
    fn shortcut(&self) -> egui::KeyboardShortcut {
        egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::Comma)
    }

    fn icon(&self) -> MaterialIcon { icons::ICON_SETTINGS }

    fn text_label(&self) -> String { "Settings".into() }

    type Params<'w, 's> = Commands<'w, 's>;

    fn precondition(&self, _: &Commands) -> bool { true }

    fn execute(&self, commands: &mut Commands) {
        commands.queue(DockCommand(|dock| {
            dock.focus_or_create(
                || Tab.into(),
                dock::ReplaceTab(|state| state.tab.is_settings()).or_always(dock::NewWindow),
            );
        }));
    }
}
