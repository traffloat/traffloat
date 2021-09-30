//! The scenraio viewer/editor.

use std::rc::Rc;

use traffloat::{def, save};
use yew::prelude::*;

use crate::app::route::*;

pub mod building;
pub mod cargo;
pub mod nav;

const SIDEBAR_WIDTH_PX: u32 = 200;
const SIDEBAR_PADDING_PX: u32 = 10;
const MAIN_WIDTH_PX: u32 = 750;

/// Displays an editor for ducts in an edge.
pub struct Comp {
    props: Props,
    link:  ComponentLink<Self>,
    file:  Option<Rc<save::SaveFile>>,
    state: State,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        let file = match save::parse(&props.buf) {
            Ok(file) => Rc::new(file),
            Err(err) => {
                props.close_hook.emit(Some(format!("Error reading save file: {}", err)));
                return Self {
                    props,
                    link,
                    file: None, // this value shouldn't be used anyway.
                    state: State::default(),
                };
            }
        };

        let mut state = State::default();
        if let Some(switch) = props.intent_route.as_ref().and_then(Switch::from_route) {
            state.switch = switch;
        }

        let ret = Self { props, link, file: Some(file), state };
        ret.state.switch.replace_state(&ret.props.name);
        ret
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::EditorHome => {
                self.state.switch = Switch::Home;
                self.state.switch.replace_state(&self.props.name);
                true
            }
            Msg::ChooseBuilding(id) => {
                self.state.switch = Switch::Building(id);
                self.state.switch.replace_state(&self.props.name);
                true
            }
            Msg::ChooseCargo(id) => {
                self.state.switch = Switch::Cargo(id);
                self.state.switch.replace_state(&self.props.name);
                true
            }
        }
    }

    fn change(&mut self, _: Props) -> ShouldRender {
        unimplemented!("I don't know when editor::Comp would be re-rendered")
    }

    fn view(&self) -> Html {
        html! {
            <>
                <nav::Comp
                    file=Rc::clone(self.file.as_ref().expect("Error case is unreachable"))
                    editor_home=self.link.callback(|()| Msg::EditorHome)
                    choose_building=self.link.callback(Msg::ChooseBuilding)
                    choose_cargo=self.link.callback(Msg::ChooseCargo)
                    route_prefix=self.route_prefix()
                    />
                <main style=style!(
                    "margin-left": format!("{}px", SIDEBAR_WIDTH_PX + SIDEBAR_PADDING_PX),
                    "border-left": "1px solid",
                    "padding": "5px 10px",
                    "height": "100vh",
                    "font-family": "'Helvetica', 'Arial', sans-serif",
                )>
                    <div style=style!(
                        "margin-left": "auto",
                        "margin-right": "auto",
                        "max-width": format!("{}px", MAIN_WIDTH_PX),
                        "overflow": "auto",
                    )>
                        { self.switch() }
                    </div>
                </main>
            </>
        }
    }
}

impl Comp {
    fn switch(&self) -> Html {
        match &self.state.switch {
            Switch::Home => html! {
                <p>
                    { "Use buttons in the navbar to view/edit details." }
                </p>
            },
            Switch::Building(building_id) => html! {
                <building::detail::Comp
                    file=Rc::clone(self.file.as_ref().expect("Error case is unreacahable"))
                    building_id=building_id.clone()
                    />
            },
            Switch::Cargo(cargo_id) => html! {
                <cargo::detail::Comp
                    file=Rc::clone(self.file.as_ref().expect("Error case is unreacahable"))
                    cargo_id=cargo_id.clone()
                    />
            },
        }
    }

    fn route_prefix(&self) -> String {
        match self.props.name.as_ref() {
            Some(name) => format!("scenario/{}", name),
            None => String::from("custom"),
        }
    }
}

/// The `Default`-initialized state of the component.
#[derive(Default)]
pub struct State {
    switch: Switch,
}

/// The mux of the main panel.
pub enum Switch {
    /// Home page for the editor.
    Home,
    /// Information for a building.
    Building(def::building::TypeId),
    /// Information for a cargo.
    Cargo(def::cargo::TypeId),
}

impl Switch {
    pub fn replace_state(&self, name: &Option<String>) {
        let rules = match self {
            Self::Home => Rules::Home,
            Self::Building(id) => Rules::Building(id.clone()),
            Self::Cargo(id) => Rules::Cargo(id.clone()),
        };
        let sp = SpRoute::Rules(rules);
        let route = match name.as_ref() {
            Some(name) => Route::Scenario { name: name.to_string(), sp },
            None => Route::Custom { sp },
        };
        route.replace_state();
    }

    pub fn from_route(route: &Route) -> Option<Self> {
        let rules = match route {
            Route::Scenario { sp: SpRoute::Rules(rules), .. } => rules,
            Route::Custom { sp: SpRoute::Rules(rules) } => rules,
            _ => return None,
        };
        Some(match rules {
            Rules::Home => Self::Home,
            Rules::Building(id) => Self::Building(id.clone()),
            Rules::Cargo(id) => Self::Cargo(id.clone()),
        })
    }
}

impl Default for Switch {
    fn default() -> Self { Self::Home }
}

/// Events for [`Comp`].
pub enum Msg {
    /// Set the main body to home.
    EditorHome,
    /// Set the main body to a building.
    ChooseBuilding(def::building::TypeId),
    /// Set the main body to a cargo.
    ChooseCargo(def::cargo::TypeId),
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// Name of the scenario, if it is default.
    pub name:         Option<String>,
    /// Buffer storing the tsv buffer.
    pub buf:          Rc<[u8]>,
    /// Callback to return to home.
    pub close_hook:   Callback<Option<String>>,
    /// The intended route to navigate to.
    pub intent_route: Option<Route>,
}
