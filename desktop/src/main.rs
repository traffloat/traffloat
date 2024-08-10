//! Binary for the desktop client app.

use bevy::app::{self, App, AppExit, PluginGroup};
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
    #[cfg(not(target_family = "wasm32"))]
    let options = Options::parse();

    App::new()
        .insert_resource(options) // inserted the earliest to allow plugins to read during build
        .add_plugins((
            bevy::DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    name: Some("Traffloat".into()),
                    title: "Traffloat".into(),
                    ..<_>::default()
                }),
                ..Default::default()
            }),
            traffloat_base::save::Plugin,
            traffloat_view::Plugin,
            traffloat_graph::Plugin,
            traffloat_fluid::Plugin(AppState::GameView),
        ))
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
