//! Binary for the desktop client app.

use bevy::app::{self, App, AppExit, PluginGroup};
use bevy::ecs::schedule::{self, ScheduleBuildSettings};
use bevy::state::app::AppExtStates;
use bevy::state::state::States;
use bevy::window::{Window, WindowPlugin};
use bevy::winit::WinitSettings;

mod main_menu;
mod util;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    MainMenu,
}

fn main() -> AppExit {
    App::new()
        .add_plugins((bevy::DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                name: Some("Traffloat".into()),
                title: "Traffloat".into(),
                ..<_>::default()
            }),
            ..Default::default()
        }),))
        .insert_resource(WinitSettings::desktop_app())
        .init_state::<AppState>()
        .add_plugins(main_menu::Plugin)
        .edit_schedule(app::Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: schedule::LogLevel::Warn,
                ..<_>::default()
            });
        })
        .run()
}
