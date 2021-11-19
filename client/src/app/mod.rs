use std::rc::Rc;

use safety::Safety;
use traffloat::{def, save, SetupEcs};

mod editor;
mod game;
mod home;
pub mod lang;
mod mux;
pub mod options;
pub mod route;
mod scenarios;

pub use mux::Mux;

use crate::util::high_res_time;
use crate::ContextPath;

/// Arguments for starting a game
#[derive(Clone)]
pub enum GameArgs {
    /// Singleplayer mode
    Sp(SpGameArgs),
}

impl GameArgs {
    /// Initializes ECS setup.
    pub fn init(&self, mut setup: SetupEcs) -> SetupEcs {
        match self {
            Self::Sp(args) => {
                setup = save::load_scenario(setup, &args.scenario, high_res_time().homosign())
                    .resource(ContextPath::new(args.context_path.to_string()));
            }
        }
        setup
    }
}

impl yew::html::ImplicitClone for GameArgs {}

/// Parameters for starting a game
#[derive(Clone)]
pub struct SpGameArgs {
    /// The scenario file.
    pub scenario:     Rc<def::TfsaveFile>,
    /// Context path of the scenario file.
    pub context_path: String,
}
