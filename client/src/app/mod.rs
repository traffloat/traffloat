mod game;
mod home;
mod mux;

pub use mux::Mux;

#[derive(Debug, Clone)]
pub enum GameArgs {
    Sp(SpGameArgs),
}

impl yew::html::ImplicitClone for GameArgs {}

#[derive(Debug, Clone)]
pub struct SpGameArgs {}
