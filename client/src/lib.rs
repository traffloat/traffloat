use bevy::app::{App, AppExit, PluginGroup};
use bevy::log::LogPlugin;
use bevy::log::tracing_subscriber::fmt::format::FmtSpan;
use bevy::log::tracing_subscriber::{self, Layer};
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy_egui::EguiPlugin;

mod dock;
mod scene;
mod util;

pub fn run() -> AppExit {
    let mut app = App::new();
    app.add_plugins(bevy::DefaultPlugins.set(LogPlugin {
        fmt_layer: |_| {
            Some(Box::new(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE)))
        },
        ..Default::default()
    }));
    app.add_plugins(MeshPickingPlugin);
    app.add_plugins(EguiPlugin::default());
    app.add_plugins(traffloat_physics::Plug);
    app.add_plugins((util::shapes::Plug, dock::Plug, scene::Plug));
    app.run()
}

pub type ConfigManager = (bevy_mod_config::manager::Egui,);
