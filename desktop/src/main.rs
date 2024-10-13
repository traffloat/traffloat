//! Binary for the desktop client app.

use bevy::app::{self, App, AppExit, PluginGroup};
use bevy::asset::AssetPlugin;
use bevy::ecs::schedule::{self, ScheduleBuildSettings};
use bevy::state::app::AppExtStates;
use bevy::state::state::States;
use bevy::window::{Window, WindowPlugin};
use bevy::winit::WinitSettings;
use bevy_mod_picking::DefaultPickingPlugins;
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
    let options = match Options::parse_by_platform() {
        Ok(options) => options,
        Err(err) => {
            eprintln!("CLI error: {err}");
            return AppExit::error();
        }
    };

    App::new()
        .add_plugins((
            bevy::DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        name: Some("Traffloat".into()),
                        title: "Traffloat".into(),
                        ..Default::default()
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
                    ..Default::default()
                }),
            DefaultPickingPlugins,
            traffloat_base::save::Plugin,
            traffloat_view::Plugin,
            traffloat_graph::Plugin,
            traffloat_fluid::Plugin(AppState::GameView),
        ))
        .insert_resource(options) // inserted the earliest to allow plugins to read during build
        .init_resource::<WinitSettings>()
        .init_state::<AppState>()
        .add_plugins((
            #[cfg(feature = "inspector")]
            bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
        ))
        .add_plugins(main_menu::Plugin)
        .add_plugins(view::Plugin)
        .edit_schedule(app::Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: schedule::LogLevel::Warn,
                ..Default::default()
            });
        })
        .run()
}
