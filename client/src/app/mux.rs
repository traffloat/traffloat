use std::rc::Rc;

use yew::prelude::*;

use super::*;

/// Wrapper component for the site.
pub struct Mux {
    link: ComponentLink<Self>,
    state: State,
}

impl Component for Mux {
    type Message = Msg;
    type Properties = ();

    fn create((): (), link: ComponentLink<Self>) -> Self {
        Self {
            link,
            state: State::Home { error: None },
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::StartSingle(args) => {
                self.state = State::Game(GameArgs::Sp(args));
                true
            }
            Msg::EditScenario(scenario) => {
                self.state = State::Editor(scenario);
                true
            }
            Msg::Exit(error) => {
                self.state = State::Home { error };
                true
            }
        }
    }

    fn change(&mut self, (): ()) -> ShouldRender {
        unreachable!()
    }

    fn view(&self) -> Html {
        match &self.state {
            State::Home { error } => html! {
                <home::Home
                    start_single_hook=self.link.callback(Msg::StartSingle)
                    edit_scenario_hook=self.link.callback(Msg::EditScenario)
                    error=error.clone() />
            },
            State::Game(args) => html! {
                <game::Game args=args error_hook=self.link.callback(Msg::Exit) />
            },
            State::Editor(buf) => html! {
                <editor::Comp buf=Rc::clone(buf) close_hook=self.link.callback(Msg::Exit) />
            },
        }
    }
}

/// Switches the component state.
pub enum Msg {
    /// Starts a singleplayer game.
    StartSingle(SpGameArgs),
    /// Edit a scenario..
    EditScenario(Rc<[u8]>),
    /// Ends a game with an optional error message.
    Exit(Option<String>),
}

enum State {
    Home { error: Option<String> },
    Game(GameArgs),
    Editor(Rc<[u8]>),
}
