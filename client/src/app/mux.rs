use yew::prelude::*;

use super::*;

pub struct Mux {
    props: Props,
    link: ComponentLink<Self>,
    state: State,
}

impl Component for Mux {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        Self {
            props,
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

pub enum Msg {
    StartSingle(SpGameArgs),
    EndGame(Option<String>),
}
type Props = ();

enum State {
    Home { error: Option<String> },
    Game(GameArgs),
}
