use std::fs;
use std::path::PathBuf;

use bevy::app::{self, App};
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, ResMut, Resource};
use bevy::ecs::world::Command;
use bevy::state::app::AppExtStates;
use bevy::state::condition::in_state;
use bevy::state::state::{self, NextState, States};
use bevy::tasks::{block_on, poll_once, IoTaskPool, Task};
use traffloat_base::save;

use crate::options::Options;
use crate::util::{modal, ui_style};
use crate::AppState;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, States)]
pub enum ActiveState {
    #[default]
    Inactive,
    Active,
}

pub struct Plugin;

impl app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        if let Some(save_file) = app.world().resource::<Options>().save_file.clone() {
            app.insert_state(ActiveState::Active);
            app.insert_resource(PreSelectedFile(Some(save_file)));
        } else {
            app.insert_state(ActiveState::Inactive);
            app.insert_resource(PreSelectedFile(None));
        }

        app.add_plugins(modal::Plugin::<ErrorButtons>::default());
        app.add_systems(state::OnEnter(ActiveState::Active), setup);
        app.add_systems(
            app::Update,
            poll_task.ambiguous_with(super::handle_click).run_if(in_state(ActiveState::Active)),
        );
        app.init_resource::<SelectFileTask>();
    }
}

#[derive(Resource)]
struct PreSelectedFile(Option<PathBuf>);

#[derive(Default, Resource)]
struct SelectFileTask(Option<Task<Option<FileSelection>>>);

struct FileSelection {
    path:     PathBuf,
    contents: std::io::Result<Vec<u8>>,
}

fn setup(mut task_res: ResMut<SelectFileTask>, mut pre_selected_file: ResMut<PreSelectedFile>) {
    let pre_selected_file = pre_selected_file.0.take();
    let pool = IoTaskPool::get_or_init(<_>::default);
    let task = pool.spawn(async {
        if let Some(path) = pre_selected_file {
            let contents = fs::read(&path);

            Some(FileSelection { path, contents })
        } else {
            let handle = rfd::AsyncFileDialog::new()
                .add_filter("Traffloat save files", &["tfsave"])
                .pick_file()
                .await?;

            let path = handle.path().to_path_buf();
            let contents = handle.read().await;
            Some(FileSelection { path, contents: Ok(contents) })
        }
    });
    task_res.0 = Some(task);
}

fn poll_task(
    mut task_res: ResMut<SelectFileTask>,
    mut active_state: ResMut<NextState<ActiveState>>,
    mut commands: Commands,
) {
    let Some(task) = task_res.0.as_mut() else { return };
    let Some(result) = block_on(poll_once(task)) else { return };

    task_res.0 = None;

    let Some(result) = result else {
        active_state.set(ActiveState::Inactive);
        return;
    };

    match result.contents {
        Ok(contents) => {
            bevy::log::info!("loaded {:?} with {} bytes", result.path, contents.len());

            commands.push(save::LoadCommand {
                data:        contents,
                on_complete: Box::new(|world, result| match result {
                    Ok(()) => {
                        world.resource_mut::<NextState<AppState>>().set(AppState::GameView);
                    }
                    Err(err) => {
                        bevy::log::error!("load error: {err:?}");
                        world.resource_mut::<NextState<ActiveState>>().set(ActiveState::Inactive);
                        modal::DisplayCommand::<ErrorButtons>::builder()
                            .background_color(ui_style::ERROR_COLOR)
                            .title("Load error")
                            .text(err.to_string())
                            .build()
                            .apply(world);
                    }
                }),
            });
        }
        Err(err) => {
            bevy::log::error!("read error: {err:?}");
            active_state.set(ActiveState::Inactive);
            commands.push(
                modal::DisplayCommand::<ErrorButtons>::builder()
                    .background_color(ui_style::ERROR_COLOR)
                    .title("Load error")
                    .text(format!("Error reading {}: {err}", result.path.display()))
                    .build(),
            );
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ErrorButtons;

impl modal::Buttons for ErrorButtons {
    fn iter() -> impl Iterator<Item = Self> { [Self].into_iter() }

    fn label(&self) -> String { "OK".into() }
}
