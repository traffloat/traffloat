use std::rc::Rc;

use yew::prelude::*;
use yew::services::fetch;
use yew::services::reader;

use super::route::{Route, SpRoute};
use super::scenarios;
use super::SpGameArgs;

mod scenario_choose;

/// The homepage for selecting gamemode.
pub struct Home {
    props: Props,
    link: ComponentLink<Self>,
    game_mode: GameMode,
    _loader: Option<ScenarioLoader>,
    scenario: Option<Rc<[u8]>>,
    chosen_scenario_name: Option<String>,
}

impl Component for Home {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        let (game_mode, chosen_scenario_name) = match &props.intent_route {
            Some(Route::Scenario { name, .. }) => (GameMode::Single, Some(name.into())),
            Some(Route::Custom { .. }) => (GameMode::Single, None),
            Some(Route::Server) => (GameMode::Multi, None),
            None => (GameMode::Single, Some("vanilla".into())),
        };
        Self {
            props,
            link,
            game_mode,
            _loader: None,
            scenario: None,
            chosen_scenario_name,
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
            Msg::ChooseScenario(event) => {
                let loader = match event.scenario {
                    Some(Scenario::Url(url)) => {
                        log::debug!("URL: {}", url);
                        let request = fetch::Request::get(url)
                            .body(yew::format::Nothing)
                            .expect("Failed to build request");
                        log::debug!("Fetching scenario from URL: {}", url);
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
                        log::debug!("Reading scenario from uploaded file: {}", file.name());
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

                if event.explicit {
                    let route = match event.name.as_ref() {
                        Some(name) => Route::Scenario {
                            name: name.to_string(),
                            sp: SpRoute::Home,
                        },
                        None => Route::Custom { sp: SpRoute::Home },
                    };
                    route.replace_state();
                }

                self.chosen_scenario_name = event.name;
                true
            }
            Msg::ScenarioUrlLoaded(resp) => {
                let _loader = self._loader.take(); // drop when function return
                let (meta, body) = resp.into_parts();
                if !meta.status.is_success() {
                    log::error!("Error fetching URL: {:?}", meta);
                    return true;
                }
                match body {
                    Ok(body) => {
                        let body = Rc::from(body);
                        self.scenario = Some(body);
                        if let Some(Route::Scenario {
                            sp: SpRoute::Game, ..
                        }) = &self.props.intent_route
                        {
                            self.link.send_message(Msg::StartSingle);
                        } else if let Some(Route::Scenario {
                            sp: SpRoute::Rules(_),
                            ..
                        }) = &self.props.intent_route
                        {
                            self.link.send_message(Msg::EditScenario);
                        }
                    }
                    Err(err) => {
                        log::error!("Error reading scenario data: {:?}", err);
                    }
                }
                true
            }
            Msg::ScenarioFileLoaded(file) => {
                self._loader = None;
                let body = Rc::from(file.content);
                self.scenario = Some(body);
                true
            }
            Msg::StartSingle => {
                let scenario = match &self.scenario {
                    Some(scenario) => Rc::clone(scenario),
                    None => return false,
                };
                self.props
                    .start_single_hook
                    .emit((SpGameArgs { scenario }, self.chosen_scenario_name.clone()));
                false
            }
            Msg::EditScenario => {
                let scenario = match &self.scenario {
                    Some(scenario) => Rc::clone(scenario),
                    None => return false,
                };
                self.props
                    .edit_scenario_hook
                    .emit((self.chosen_scenario_name.clone(), scenario));
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
                            choose_scenario=self.link.callback(Msg::ChooseScenario )
                            intent_route=self.props.intent_route.clone()
                            />
                        <div>
                            <button
                                onclick=self.link.callback(|_| Msg::StartSingle)
                                disabled=self.scenario.is_none()
                                tabindex=1 >
                                { "Start" }
                            </button>
                            <button
                                onclick=self.link.callback(|_| Msg::EditScenario)
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
    ChooseScenario(ChooseScenario),
    /// Scenario file has been uploaded.
    ScenarioFileLoaded(reader::FileData),
    /// Scenario URL has been downloaded.
    ScenarioUrlLoaded(fetch::Response<yew::format::Binary>),
    /// Starts a singleplayer game.
    StartSingle,
    /// Edit a scenario.
    EditScenario,
}

/// An event of choosing a scenario.
pub struct ChooseScenario {
    /// The source of scenario.
    pub scenario: Option<Scenario>,
    /// The name of the scenario.
    pub name: Option<String>,
    /// Whether the scenario was explicit chosen
    /// (`false` if inferred from URL).
    pub explicit: bool,
}

/// yew properties for [`Home`][Home].
#[derive(Clone, Properties)]
pub struct Props {
    /// Callback to start a singleplayer game.
    pub start_single_hook: Callback<(SpGameArgs, Option<String>)>,
    /// Callback to edit a scenario.
    pub edit_scenario_hook: Callback<(Option<String>, Rc<[u8]>)>,
    /// The intended route to navigate to.
    pub intent_route: Option<Route>,
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
            scenarios::OPTIONS
                .get(0)
                .expect("scenarios::OPTIONS is empty")
                .path,
        )
    }
}

enum ScenarioLoader {
    Url(fetch::FetchTask),
    File(reader::ReaderTask),
}
