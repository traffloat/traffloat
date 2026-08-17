use std::panic;
use std::path::PathBuf;

use bevy::app::{self, App, AppExit, PluginGroup};
use bevy::asset::AssetPlugin;
use bevy::ecs::resource::Resource;
use bevy::log::tracing_subscriber::fmt::format::FmtSpan;
use bevy::log::{LogPlugin, tracing_subscriber};
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy_egui::EguiPlugin;

mod dock;
mod scene;
mod util;

pub fn run(options: Options) -> AppExit {
    let mut app = App::new();
    app.insert_resource(options.clone());
    app.add_plugins(
        bevy::DefaultPlugins
            .set(LogPlugin {
                fmt_layer: move |app| {
                    let options = app.world().resource::<Options>();
                    if options.log_spans {
                        Some(Box::new(
                            tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE),
                        ))
                    } else {
                        None
                    }
                },
                custom_layer: {
                    move |app| {
                        #[cfg(feature = "otel")]
                        {
                            let options = &app.world().resource::<Options>().otel;
                            if options.otel_export {
                                use opentelemetry::trace::TracerProvider;

                                let exporter = opentelemetry_otlp::SpanExporter::builder()
                                    .with_http()
                                    .build()
                                    .expect("failed to create otel exporter");
                                let provider =
                                    opentelemetry_sdk::trace::SdkTracerProvider::builder()
                                        .with_simple_exporter(exporter)
                                        .build();
                                let tracer = provider.tracer("traffloat");
                                return Some(Box::new(
                                    tracing_opentelemetry::layer().with_tracer(tracer),
                                ));
                            }
                        }
                        None
                    }
                },
                ..Default::default()
            })
            .set(AssetPlugin { file_path: options.assets_path.clone(), ..Default::default() }),
    );
    app.add_plugins(MeshPickingPlugin);
    app.add_plugins(EguiPlugin::default());
    #[cfg(feature = "dev")]
    app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::default());
    app.add_plugins(traffloat_physics::Plug);
    app.add_plugins((util::shapes::Plug, dock::Plug, scene::Plug));
    app.add_systems(app::PreUpdate, || tracing::trace!("pre update"));
    app.add_systems(app::PostUpdate, || tracing::trace!("post update"));

    #[cfg(not(target_arch = "wasm32"))]
    if let Err(err) = handle_direct_startup(&mut app) {
        tracing::error!("Invalid arguments: {err}");
        return AppExit::from_code(1);
    };

    let mut app_unwind = panic::AssertUnwindSafe(&mut app);
    let result = panic::catch_unwind(move || app_unwind.run());
    match result {
        Ok(AppExit::Success) => AppExit::Success,
        Ok(failure) => {
            traffloat_physics::util::panic_dump(app.world());
            failure
        }
        Err(err) => {
            traffloat_physics::util::panic_dump(app.world());
            panic::resume_unwind(err);
        }
    }
}

#[derive(Clone, Resource, clap::Parser)]
pub struct Options {
    #[clap(long, env, default_value = "./assets")]
    pub assets_path: String,

    #[clap(long, env)]
    pub log_spans: bool,
    #[cfg(feature = "otel")]
    #[clap(flatten)]
    pub otel:      OtelOptions,

    #[clap(long)]
    pub new_level: bool,

    pub load_level: Option<PathBuf>,
}

#[cfg(feature = "otel")]
#[derive(clap::Args, Clone)]
pub struct OtelOptions {
    /// If specified, enables an OTLP otel exporter.
    #[clap(long, env)]
    pub otel_export:  bool,
    /// Whether to batch otel exporter.
    #[clap(long, env)]
    pub otel_batched: bool,
}

pub type ConfigManager = (bevy_mod_config::manager::Egui,);

#[cfg(not(target_arch = "wasm32"))]
fn handle_direct_startup(app: &mut App) -> Result<(), String> {
    use bevy::ecs::system::Commands;
    use traffloat_physics::generate;

    let options = app.world().resource::<Options>().clone();

    if options.new_level {
        app.add_systems(app::PostStartup, |mut commands: Commands| {
            commands.queue(dock::new_level::NewGameCommand { config: generate::Config::default() })
        });
    }

    if let Some(path) = options.load_level {
        app.add_systems(app::PostStartup, move |mut commands: Commands| {
            commands.queue(dock::save::LoadPathCommand { path: path.clone() })
        });
    }

    Ok(())
}
