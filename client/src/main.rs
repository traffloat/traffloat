use bevy::app::AppExit;

fn main() -> AppExit { traffloat_client::run(<traffloat_client::Options as clap::Parser>::parse()) }
