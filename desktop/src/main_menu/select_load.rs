use std::path::PathBuf;

use bevy::app::{self, App};
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{ResMut, Resource};
use bevy::state::app::AppExtStates;
use bevy::state::condition::in_state;
use bevy::state::state::{self, NextState, States};
use bevy::tasks::{block_on, poll_once, IoTaskPool, Task};

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
        app.add_systems(state::OnEnter(ActiveState::Active), setup);
        app.add_systems(app::Update, poll_task.run_if(in_state(ActiveState::Active)));
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
        let handle = rfd::AsyncFileDialog::new().pick_file().await?;

        let path = handle.path().to_path_buf();
        let contents = handle.read().await;

        Some(FileSelection { path, contents })
    });
    task_res.0 = Some(task);
}

fn poll_task(
    mut task_res: ResMut<SelectFileTask>,
    mut active_state: ResMut<NextState<ActiveState>>,
) {
    if let Some(task) = task_res.0.as_mut() {
        if let Some(result) = block_on(poll_once(task)) {
            task_res.0 = None;

            match result {
                None => {
                    active_state.set(ActiveState::Inactive);
                }
                Some(result) => {
                    // TODO parse
                    eprintln!("got {:?} with {} bytes", result.path, result.contents.len());
                }
            }
        }
    }
}
