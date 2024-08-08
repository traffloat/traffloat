use std::path::PathBuf;

use bevy::app::{self, App};
use bevy::color::Color;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, ResMut, Resource};
use bevy::ecs::world::Command;
use bevy::state::app::AppExtStates;
use bevy::state::condition::in_state;
use bevy::state::state::{self, NextState, States};
use bevy::tasks::{block_on, poll_once, IoTaskPool, Task};
use traffloat_base::save;

use crate::util::modal;
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
        app.init_state::<ActiveState>();
        app.add_plugins(modal::Plugin::<ErrorButtons>::default());
        app.add_systems(state::OnEnter(ActiveState::Active), setup);
        app.add_systems(
            app::Update,
            poll_task.ambiguous_with(super::handle_click).run_if(in_state(ActiveState::Active)),
        );
        app.init_resource::<SelectFileTask>();
    }
}

#[derive(Default, Resource)]
struct SelectFileTask(Option<Task<Option<FileSelection>>>);

struct FileSelection {
    path:     PathBuf,
    contents: Vec<u8>,
}

fn setup(mut task_res: ResMut<SelectFileTask>) {
    let pool = IoTaskPool::get_or_init(<_>::default);
    let task = pool.spawn(async {
        let handle = rfd::AsyncFileDialog::new()
            .add_filter("Traffloat save files", &["tfsave"])
            .pick_file()
            .await?;

        let path = handle.path().to_path_buf();
        let contents = handle.read().await;

        Some(FileSelection { path, contents })
    });
    task_res.0 = Some(task);
}

fn poll_task(
    mut task_res: ResMut<SelectFileTask>,
    mut active_state: ResMut<NextState<ActiveState>>,
    mut commands: Commands,
) {
    if let Some(task) = task_res.0.as_mut() {
        if let Some(result) = block_on(poll_once(task)) {
            task_res.0 = None;

            match result {
                None => {
                    active_state.set(ActiveState::Inactive);
                }
                Some(result) => {
                    bevy::log::info!(
                        "loaded {:?} with {} bytes",
                        result.path,
                        result.contents.len()
                    );

                    commands.push(save::LoadCommand {
                        data:        result.contents,
                        on_complete: Box::new(|world, result| match result {
                            Ok(()) => {
                                world.resource_mut::<NextState<AppState>>().set(AppState::GameView);
                            }
                            Err(err) => {
                                bevy::log::info!("load error: {err:?}");
                                world
                                    .resource_mut::<NextState<ActiveState>>()
                                    .set(ActiveState::Inactive);
                                modal::DisplayCommand::<ErrorButtons>::builder()
                                    .background_color(Color::srgb(0.4, 0.1, 0.1))
                                    .title("Load error")
                                    .text(err.to_string())
                                    .build()
                                    .apply(world);
                            }
                        }),
                    });
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ErrorButtons;

impl modal::Buttons for ErrorButtons {
    fn iter() -> impl Iterator<Item = Self> { [Self].into_iter() }

    fn label(&self) -> String { "OK".into() }
}
