use std::rc::Rc;

use traffloat::def;
use yew::prelude::*;

use super::route::{Route, SpRoute};
use super::*;

/// Wrapper component for the site.
pub struct Mux {
    link:         ComponentLink<Self>,
    state:        State,
    intent_route: Option<Route>,
}

impl Component for Mux {
    type Message = Msg;
    type Properties = ();

    fn create((): (), link: ComponentLink<Self>) -> Self {
        let hash = web_sys::window()
            .expect("Cannot get window")
            .location()
            .hash()
            .unwrap_or_else(|_| String::new());
        let route = Route::parse_path(&hash);
        log::debug!("Path parsed as {:?}", route);

        Self { link, state: State::Home { error: None }, intent_route: Some(route) }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::StartSingle { args, scenario } => {
                self.state = State::Game(GameArgs::Sp(args));
                let sp = SpRoute::Game;
                let route = match scenario {
                    Some(name) => Route::Scenario { name, sp },
                    None => Route::Custom { sp },
                };
                route.replace_state(None);
                true
            }
            Msg::EditScenario(name, scenario) => {
                self.state = State::Editor(name, scenario);
                true
            }
            Msg::Exit(error) => {
                self.state = State::Home { error };
                self.intent_route = None;
                true
            }
        }
    }

    fn change(&mut self, (): ()) -> ShouldRender { unreachable!() }

    fn view(&self) -> Html {
        match &self.state {
            State::Home { error } => html! {
                <home::Home
                    start_single_hook=self.link.callback(|(args, scenario)| Msg::StartSingle { args, scenario })
                    edit_scenario_hook=self.link.callback(|(name, buf)| Msg::EditScenario(name, buf))
                    intent_route=self.intent_route.clone()
                    error=error.clone()
                    />
            },
            State::Game(args) => html! {
                <game::Game
                    args=args
                    error_hook=self.link.callback(Msg::Exit)
                    />
            },
            State::Editor(name, schema) => html! {
                <editor::Comp
                    name=name.clone()
                    schema=Rc::clone(schema)
                    close_hook=self.link.callback(Msg::Exit)
                    intent_route=self.intent_route.clone()
                    />
            },
        }
    }
}

/// Switches the component state.
pub enum Msg {
    /// Starts a singleplayer game.
    StartSingle { args: SpGameArgs, scenario: Option<String> },
    /// Edit a scenario..
    EditScenario(Option<String>, Rc<def::TfsaveFile>),
    /// Ends a game with an optional error message.
    Exit(Option<String>),
}

enum State {
    Home { error: Option<String> },
    Game(GameArgs),
    Editor(Option<String>, Rc<def::TfsaveFile>),
}
