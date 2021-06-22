mod game;
mod home;
mod mux;

pub use mux::Mux;

/// Arguments for starting a game
#[derive(Debug, Clone)]
pub enum GameArgs {
    /// Singleplayer mode
    Sp(SpGameArgs),
}

impl yew::html::ImplicitClone for GameArgs {}

/// Parameters for starting a game
#[derive(Debug, Clone)]
pub struct SpGameArgs {}
