use std::time::Duration;

use bevy::app::{self, App, Plugin};
use bevy::ecs::resource::Resource;
use bevy::ecs::system::{Command, Commands, Res, ResMut, SystemParam};
use bevy::ecs::world::World;
use bevy::state::state::State as BevyState;
use egui_material_icons::{MaterialIcon, icons};
use traffloat_physics::persist;

use crate::dock::{self, TabPlacement, menu};
use crate::scene::{self, LevelState};
use crate::util::new_id;

pub mod storage;
pub use storage::Storage;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadSource>();
        app.init_resource::<Storage>();
        app.add_systems(app::PostUpdate, |world: &mut World| {
            let mut flush = world.resource_mut::<Storage>().0.flush();
            flush.apply(world);
        });
    }
}

#[derive(Resource, Default)]
pub struct LoadSource(pub Option<LoadSourceInner>);

pub struct LoadSourceInner {
    pub name: String,
}

#[derive(Default)]
pub struct OpenTab {
    pub name_edit: String,
}

impl dock::Tab for OpenTab {
    type TitleSystemParam<'w, 's> = ();
    fn title(&self, param: Self::TitleSystemParam<'_, '_>) -> String { "Open".into() }

    type UiSystemParam<'w, 's> = OpenUiSystemParam<'w, 's>;
    fn ui(
        &mut self,
        mut params: Self::UiSystemParam<'_, '_>,
        ui: &mut egui::Ui,
        ctx: dock::Context,
    ) {
        let mut entries = params.storage.0.list_entries();
        entries.sort_by(storage::SortBy::LastModified.sort_fn());

        ui.horizontal(|ui| {
            egui::ComboBox::new(new_id!(ctx.id), "").selected_text(&self.name_edit).show_ui(
                ui,
                |ui| {
                    for entry in entries {
                        ui.selectable_value(
                            &mut self.name_edit,
                            entry.name.clone(),
                            entry.name.clone(),
                        );
                    }
                },
            );

            if ui.button(icons::ICON_REFRESH).on_hover_text("Refresh list").clicked() {
                params.storage.0.reload();
            }
        });

        let is_valid = params.storage.0.is_name_used(&self.name_edit);
        ui.add_enabled_ui(is_valid, |ui| {
            if ui.button("OK").clicked() && is_valid {
                params.commands.queue(LoadCommand { name: self.name_edit.clone() });
            }
        });
    }

    type OnCloseSystemParam<'w, 's> = ();

    type BeforeRenderSystemParam<'w, 's> = ();
}

#[derive(SystemParam)]
pub struct OpenUiSystemParam<'w, 's> {
    commands: Commands<'w, 's>,
    storage:  ResMut<'w, Storage>,
}

pub struct SaveAsTab {
    name_edit:       Option<String>,
    set_as_default:  bool,
    allow_duplicate: bool,
}

impl Default for SaveAsTab {
    fn default() -> Self { Self { name_edit: None, set_as_default: true, allow_duplicate: false } }
}

impl dock::Tab for SaveAsTab {
    type TitleSystemParam<'w, 's> = ();
    fn title(&self, param: Self::TitleSystemParam<'_, '_>) -> String { "Save as".into() }

    type UiSystemParam<'w, 's> = SaveAsUiSystemParam<'w, 's>;
    fn ui(
        &mut self,
        mut params: Self::UiSystemParam<'_, '_>,
        ui: &mut egui::Ui,
        ctx: dock::Context,
    ) {
        let mut is_name_used = false;
        let name_edit = self.name_edit.get_or_insert_with(|| {
            params.load_source.0.as_ref().map_or("", |s| &s.name).to_string()
        });
        let mut initiated_submit = false;

        ui.horizontal(|ui| {
            ui.label("Name:");

            let resp = ui.text_edit_singleline(name_edit);
            is_name_used = params.storage.0.is_name_used(name_edit.as_str());
            if resp.changed() {
                self.allow_duplicate = !is_name_used;
            }
            if resp.lost_focus() && resp.ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                initiated_submit = true;
            }
        });

        if params.load_source.0.is_some() {
            ui.checkbox(&mut self.set_as_default, "Use this name for future saves");
        }

        ui.add_enabled_ui(is_name_used, |ui| {
            if is_name_used {
                ui.checkbox(&mut self.allow_duplicate, "Overwrite existing save");
            } else {
                ui.checkbox(&mut true, "Create new save");
            }
        });

        ui.add_enabled_ui(is_valid_name(name_edit) && self.allow_duplicate, |ui| {
            if ui.button("OK").clicked() || initiated_submit {
                params.commands.queue(SaveCommand {
                    name:           name_edit.clone(),
                    set_as_default: self.set_as_default,
                });
            }
        });
    }

    type OnCloseSystemParam<'w, 's> = ();

    type BeforeRenderSystemParam<'w, 's> = ();
}

fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains('\0')
        && !name.contains(':')
}

#[derive(SystemParam)]
pub struct SaveAsUiSystemParam<'w, 's> {
    commands:    Commands<'w, 's>,
    load_source: ResMut<'w, LoadSource>,
    storage:     ResMut<'w, Storage>,
}

pub struct LoadCommand {
    pub name: String,
}

impl Command for LoadCommand {
    type Out = ();
    #[tracing::instrument(skip_all, fields(name = self.name))]
    fn apply(self, world: &mut World) {
        tracing::info!("Reading save {}", self.name);
        let name = self.name.clone();
        world.resource_mut::<Storage>().0.load_file(
            &self.name,
            Box::new(move |world, data| {
                tracing::info!("Loading save {name} with {} bytes", data.len());
                if let Err(err) = persist::input(world, data) {
                    world
                        .resource_mut::<dock::Toasts>()
                        .0
                        .error(format!("Failed to load world: {err}"))
                        .duration(Duration::from_secs(15));
                    tracing::error!("Failed to load world: {err}");
                    return;
                }

                scene::singleplayer::setup(world);
                dock::init_camera_view(world);
            }),
        );

        let mut load_source = world.resource_mut::<LoadSource>();
        load_source.0 = Some(LoadSourceInner { name: self.name });
    }
}

pub struct SaveCommand {
    pub name:           String,
    pub set_as_default: bool,
}

impl Command for SaveCommand {
    type Out = ();
    fn apply(self, world: &mut World) {
        if let Ok(data) = persist::output(world) {
            let mut storage = world.resource_mut::<Storage>();
            storage.0.submit_save(&self.name, &data);
            world
                .resource_mut::<dock::Toasts>()
                .0
                .info(format!("Saved world as \"{}\"", self.name));

            if self.set_as_default {
                let mut load_source = world.resource_mut::<LoadSource>();
                load_source.0 = Some(LoadSourceInner { name: self.name });
            }
        } else {
            world.resource_mut::<dock::Toasts>().0.error("Failed to serialize world");
            tracing::error!("Failed to serialize world");
        }
    }
}

#[derive(Default)]
pub struct MenuActionSave;

impl menu::Action for MenuActionSave {
    fn shortcut(&self) -> egui::KeyboardShortcut {
        egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::S)
    }

    fn icon(&self) -> MaterialIcon { icons::ICON_SAVE }

    fn text_label(&self) -> String { "Save".into() }

    type Params<'w, 's> = MenuActionParams<'w, 's>;

    fn precondition(&self, params: &MenuActionParams) -> bool {
        params.level_state.is_singleplayer() && params.src.0.is_some()
    }

    fn execute(&self, params: &mut MenuActionParams) {
        if let Some(src) = &params.src.0 {
            params
                .commands
                .queue(SaveCommand { name: src.name.clone(), set_as_default: false });
        }
    }
}

#[derive(Default)]
pub struct MenuActionSaveAs;

impl menu::Action for MenuActionSaveAs {
    fn shortcut(&self) -> egui::KeyboardShortcut {
        egui::KeyboardShortcut::new(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT, egui::Key::S)
    }

    fn icon(&self) -> MaterialIcon { icons::ICON_SAVE_AS }

    fn text_label(&self) -> String { "Save as".into() }

    type Params<'w, 's> = MenuActionParams<'w, 's>;

    fn precondition(&self, params: &MenuActionParams) -> bool {
        params.level_state.is_singleplayer()
    }

    fn execute(&self, params: &mut MenuActionParams) {
        params.commands.queue(dock::DockCommand(|dock| {
            dock.focus_or_create(
                || SaveAsTab::default().into(),
                dock::ReplaceTab(|state| state.tab.is_save_as()).or_always(dock::NewWindow),
            );
        }));
    }
}

#[derive(SystemParam)]
pub struct MenuActionParams<'w, 's> {
    commands:    Commands<'w, 's>,
    src:         Res<'w, LoadSource>,
    level_state: Res<'w, BevyState<LevelState>>,
}
