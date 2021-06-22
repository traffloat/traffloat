use yew::prelude::*;

use super::SpGameArgs;

/// The homepage for selecting gamemode.
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
            <div style="margin: 0 auto; max-width: 720px;">
                <h1>{ "Traffloat" }</h1>

                { for self.props.error.as_ref().map(|error| html! {
                    <div>
                        { "Error: " }
                        <span>{ error }</span>
                    </div>
                }) }

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

/// Messages for updating the user interface.
pub enum Msg {
    /// Selects the single player mode.
    ModeSingle(MouseEvent),
    /// Selects the multi player mode.
    ModeMulti(MouseEvent),
    /// Starts a singleplayer game.
    StartSingle(MouseEvent),
}

/// yew properties for [`Home`][Home].
#[derive(Clone, Properties)]
pub struct Props {
    /// Callback to start a singleplayer game.
    pub start_single_hook: Callback<SpGameArgs>,
    /// Displays an error message.
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameMode {
    Single,
    Multi,
}
