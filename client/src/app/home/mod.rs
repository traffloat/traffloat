use std::rc::Rc;

use yew::prelude::*;
use yew::services::fetch;
use yew::services::reader;

use super::SpGameArgs;

mod scenario_choose;

/// The homepage for selecting gamemode.
pub struct Home {
    props: Props,
    link: ComponentLink<Self>,
    game_mode: GameMode,
    _loader: Option<ScenarioLoader>,
    scenario: Option<Rc<[u8]>>,
}

impl Component for Home {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            game_mode: GameMode::Single,
            _loader: None,
            scenario: None,
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
            Msg::ChooseScenario(scenario) => {
                let loader = match scenario {
                    Some(Scenario::Url(url)) => {
                        let request = fetch::Request::get(url)
                            .body(yew::format::Nothing)
                            .expect("Failed to build request");
                        match fetch::FetchService::fetch_binary(
                            request,
                            self.link.callback(Msg::ScenarioUrlLoaded),
                        ) {
                            Ok(task) => Some(ScenarioLoader::Url(task)),
                            Err(err) => {
                                log::error!("{:?}", err);
                                // TODO display to user
                                None
                            }
                        }
                    }
                    Some(Scenario::File(file)) => {
                        match reader::ReaderService::read_file(
                            file,
                            self.link.callback(Msg::ScenarioFileLoaded),
                        ) {
                            Ok(task) => Some(ScenarioLoader::File(task)),
                            Err(err) => {
                                log::error!("{:?}", err);
                                // TODO display to user
                                None
                            }
                        }
                    }
                    None => None,
                };
                self.scenario = None;
                self._loader = loader;
                true
            }
            Msg::ScenarioUrlLoaded(resp) => {
                self._loader = None;
                let (_meta, body) = resp.into_parts();
                // TODO handle error if !meta.is_success() or body.is_err()
                if let Ok(body) = body {
                    let body = Rc::from(body);
                    self.scenario = Some(body);
                }
                true
            }
            Msg::ScenarioFileLoaded(file) => {
                self._loader = None;
                let body = Rc::from(file.content);
                self.scenario = Some(body);
                true
            }
            Msg::StartSingle(_) => {
                let scenario = match &self.scenario {
                    Some(scenario) => Rc::clone(scenario),
                    None => return false,
                };
                self.props.start_single_hook.emit(SpGameArgs { scenario });
                false
            }
            Msg::EditScenario(_) => {
                let scenario = match &self.scenario {
                    Some(scenario) => Rc::clone(scenario),
                    None => return false,
                };
                self.props.edit_scenario_hook.emit(scenario);
                false
            }
        }
    }

    fn change(&mut self, _: Props) -> ShouldRender {
        unreachable!()
    }

    fn view(&self) -> Html {
        html! {
            <div style="margin: 0 auto; max-width: 720px; font-family: 'Helvetica', 'Arial', sans-serif;">
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
                    <>
                        <scenario_choose::Comp
                            choose_scenario=self.link.callback(Msg::ChooseScenario)
                            />
                        <div>
                            <button
                                onclick=self.link.callback(Msg::StartSingle)
                                disabled=self.scenario.is_none()
                                tabindex=1 >
                                { "Start" }
                            </button>
                            <button
                                onclick=self.link.callback(Msg::EditScenario)
                                disabled=self.scenario.is_none()
                                tabindex=2 >
                                { "Rules" }
                            </button>
                        </div>
                    </>
                }) }

                // TODO handle multiplayer

                <footer style="position: fixed; bottom: 0; left: 0; width: 100%;">
                    <ul style="text-align: center; display: block; padding: 0;">
                        { for [
                            ("Source code", "https://github.com/traffloat/traffloat"),
                            ("User manual", "https://traffloat.github.io/guide/master/"),
                            ("Discussion", "https://github.com/traffloat/traffloat/discussions"),
                        ].iter().map(|&(name, url)| html! {
                            <li style="display: inline; margin: 0.5em;"><a href=url target="_blank">{ name }</a></li>
                        }) }
                    </ul>
                    <p style="text-align: center;">{
                        format_args!( "v{}", traffloat_version::VERSION)
                    }</p>
                    <p style="text-align: center;">
                        { "Licensed under " }
                        <a href="https://www.gnu.org/licenses/agpl-3.0.en.html" target="_blank">
                            { "GNU Affero General Public License" }
                        </a>
                    </p>
                </footer>
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
    /// Chooses a singleplayer scenario.
    ChooseScenario(Option<Scenario>),
    /// Scenario file has been uploaded.
    ScenarioFileLoaded(reader::FileData),
    /// Scenario URL has been downloaded.
    ScenarioUrlLoaded(fetch::Response<yew::format::Binary>),
    /// Starts a singleplayer game.
    StartSingle(MouseEvent),
    /// Edit a scenario.
    EditScenario(MouseEvent),
}

/// yew properties for [`Home`][Home].
#[derive(Clone, Properties)]
pub struct Props {
    /// Callback to start a singleplayer game.
    pub start_single_hook: Callback<SpGameArgs>,
    /// Callback to edit a scenario.
    pub edit_scenario_hook: Callback<Rc<[u8]>>,
    /// Displays an error message.
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameMode {
    Single,
    Multi,
}

/// A selection of scenario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scenario {
    /// Load a scenario from a URL.
    Url(&'static str),
    /// Load a scenario from an uploaded file.
    File(web_sys::File),
}

impl Default for Scenario {
    fn default() -> Self {
        Self::Url(
            scenario_choose::SCENARIO_OPTIONS
                .get(0)
                .expect("SCENARIO_OPTIONS is empty")
                .1,
        )
    }
}

enum ScenarioLoader {
    Url(fetch::FetchTask),
    File(reader::ReaderTask),
}
