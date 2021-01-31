use yew::prelude::*;

use super::SpGameArgs;

pub struct Home {
    props: Props,
    link: ComponentLink<Self>,
    game_mode: GameMode,
}

impl Component for Home {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            game_mode: GameMode::Single,
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::ModeSingle(event) => {
                event.prevent_default();
                self.game_mode = GameMode::Single;
                true
            }
            Msg::ModeMulti(event) => {
                event.prevent_default();
                self.game_mode = GameMode::Multi;
                true
            }
            Msg::StartSingle(_) => {
                self.props.start_single_hook.emit(SpGameArgs {});
                false
            }
        }
    }

    fn change(&mut self, _: Props) -> ShouldRender {
        unreachable!()
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <h1>{ "Traffloat" }</h1>

                <div>
                    <ul>
                        <li
                            style={
                                if self.game_mode == GameMode::Single { "background-color:green;" }
                                else { "" }
                            }>
                            <a
                                href="#"
                                onclick=self.link.callback(Msg::ModeSingle) >
                                { "Single-player" }
                            </a>
                        </li>
                        <li
                            style={
                                if self.game_mode == GameMode::Multi { "background-color:green;" }
                                else { "" }
                            }>
                            <a
                                href="#"
                                onclick=self.link.callback(Msg::ModeMulti) >
                                { "Multi-player" }
                            </a>
                        </li>
                    </ul>
                </div>

                { for (self.game_mode == GameMode::Single).then(|| html! {
                    <div>
                        <button onclick=self.link.callback(Msg::StartSingle)>{ "Start" }</button>
                    </div>
                }) }

                // TODO handle multiplayer
            </div>
        }
    }
}

pub enum Msg {
    ModeSingle(MouseEvent),
    ModeMulti(MouseEvent),
    StartSingle(MouseEvent),
}

#[derive(Clone, Properties)]
pub struct Props {
    pub start_single_hook: Callback<SpGameArgs>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameMode {
    Single,
    Multi,
}
