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
            state: State::Home,
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::StartSingle(args) => {
                self.state = State::Game(GameArgs::Sp(args));
                true
            }
            Msg::Error(error) => todo!(),
        }
    }

    fn change(&mut self, (): Props) -> ShouldRender {
        unreachable!()
    }

    fn view(&self) -> Html {
        match &self.state {
            State::Home => html! {
                <home::Home start_single_hook=self.link.callback(Msg::StartSingle) />
            },
            State::Game(args) => html! {
                <game::Game args=args error_hook=self.link.callback(Msg::Error) />
            },
        }
    }
}

pub enum Msg {
    StartSingle(SpGameArgs),
    Error(String),
}
type Props = ();

enum State {
    Home,
    Game(GameArgs),
}
