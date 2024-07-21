use bevy::prelude::{App, AppExit, PluginGroup, States, Window};
use bevy::state::app::AppExtStates;
use bevy::window::WindowPlugin;
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
        .run()
}
