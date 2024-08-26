//! Binary for the desktop client app.

use std::fs;

use bevy::app::{self, App, AppExit, PluginGroup};
use bevy::asset::AssetPlugin;
use bevy::ecs::schedule::{self, ScheduleBuildSettings};
use bevy::state::app::AppExtStates;
use bevy::state::state::States;
use bevy::window::{Window, WindowPlugin};
use bevy::winit::WinitSettings;
use options::Options;

mod main_menu;
mod options;
mod util;
mod view;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    MainMenu,
    GameView,
}

fn main() -> AppExit {
    #[cfg(target_family = "wasm")]
    let options = Options::default();
    #[cfg(not(target_family = "wasm"))]
    let options = {
        let mut options = Options::parse();
        let asset_dir = match fs::canonicalize(&options.asset_dir) {
            Ok(asset_dir) => asset_dir,
            Err(err) => {
                eprintln!(
                    "Asset directory {} is not canonicalizable: {err}",
                    options.asset_dir.display()
                );
                return AppExit::error();
            }
        };
        options.asset_dir = asset_dir;
        options
    };

    App::new()
        .add_plugins((
            bevy::DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        name: Some("Traffloat".into()),
                        title: "Traffloat".into(),
                        ..<_>::default()
                    }),
                    ..Default::default()
                })
                .set(AssetPlugin {
                    file_path: if let Some(asset_dir) = options.asset_dir.to_str() {
                        String::from(asset_dir)
                    } else {
                        eprintln!("Asset path is not UTF-8");
                        return AppExit::error();
                    },
                    ..<_>::default()
                }),
            traffloat_base::save::Plugin,
            traffloat_view::Plugin,
            traffloat_graph::Plugin,
            traffloat_fluid::Plugin(AppState::GameView),
        ))
        .insert_resource(options) // inserted the earliest to allow plugins to read during build
        .init_resource::<WinitSettings>()
        .init_state::<AppState>()
        .add_plugins(main_menu::Plugin)
        .add_plugins(view::Plugin)
        .edit_schedule(app::Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: schedule::LogLevel::Warn,
                ..<_>::default()
            });
        })
        .run()
}
