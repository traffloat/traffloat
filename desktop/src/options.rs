use bevy::ecs::system::Resource;

#[derive(clap::Parser, Resource)]
#[command(name = "traffloat", version = traffloat_version::VERSION, about)]
pub struct Options {}

impl Options {
    pub fn parse() -> Self { <Self as clap::Parser>::parse() }
}
