//! Binary for the desktop client app.

use std::time::Duration;

use bevy::app::{self, App, AppExit, PluginGroup};
use bevy::ecs::schedule::{self, ScheduleBuildSettings};
use bevy::state::app::AppExtStates;
use bevy::state::state::States;
use bevy::window::{Window, WindowPlugin};
use bevy::winit::{self, WinitSettings};

mod main_menu;
mod util;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    MainMenu,
    InGame,
}

fn main() -> AppExit {
    App::new()
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
            traffloat_graph::Plugin,
            traffloat_fluid::Plugin(AppState::InGame),
        ))
        .insert_resource(WinitSettings {
            focused_mode:   winit::UpdateMode::reactive(Duration::from_millis(100)),
            unfocused_mode: winit::UpdateMode::reactive_low_power(Duration::from_secs(1)),
        })
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
