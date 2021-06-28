use yew::prelude::*;

use super::*;

/// Wrapper component for the site.
pub struct Mux {
    _props: Props,
    link: ComponentLink<Self>,
    state: State,
}

impl Component for Mux {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        Self {
            _props: props,
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
            Msg::EndGame(error) => {
                self.state = State::Home { error };
                true
            }
        }
    }

    fn change(&mut self, (): Props) -> ShouldRender {
        unreachable!()
    }

    fn view(&self) -> Html {
        match &self.state {
            State::Home { error } => html! {
                <home::Home start_single_hook=self.link.callback(Msg::StartSingle) error=error.clone() />
            },
            State::Game(args) => html! {
                <game::Game args=args error_hook=self.link.callback(Msg::EndGame) />
            },
        }
    }
}

/// Switches the component state.
pub enum Msg {
    /// Starts a singleplayer game.
    StartSingle(SpGameArgs),
    /// Ends a game with an optional error message.
    EndGame(Option<String>),
}
type Props = ();

enum State {
    Home { error: Option<String> },
    Game(GameArgs),
}
