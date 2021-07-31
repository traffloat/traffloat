use std::rc::Rc;

use crate::util::high_res_time;
use traffloat::SetupEcs;

mod editor;
mod game;
mod home;
pub mod icon;
mod mux;

pub use mux::Mux;

/// Arguments for starting a game
#[derive(Debug, Clone)]
pub enum GameArgs {
    /// Singleplayer mode
    Sp(SpGameArgs),
}

impl GameArgs {
    /// Initializes ECS setup.
    pub fn init(&self, mut setup: SetupEcs) -> SetupEcs {
        match self {
            Self::Sp(args) => {
                setup = match traffloat::save::load(setup, &args.scenario[..], high_res_time()) {
                    Ok(setup) => setup,
                    Err(err) => todo!("Handle error {:?}", err),
                }
            }
        }
        setup
    }
}

impl yew::html::ImplicitClone for GameArgs {}

/// Parameters for starting a game
#[derive(Debug, Clone)]
pub struct SpGameArgs {
    /// The scenario file.
    pub scenario: Rc<[u8]>,
}
