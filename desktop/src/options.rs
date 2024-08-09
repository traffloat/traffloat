use std::path::PathBuf;

use bevy::ecs::system::Resource;

#[derive(clap::Parser, Resource, Default)]
#[command(name = "traffloat", version = traffloat_version::VERSION, about)]
pub struct Options {
    pub save_file: Option<PathBuf>,
}

impl Options {
    pub fn parse() -> Self { <Self as clap::Parser>::parse() }
}
