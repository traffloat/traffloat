//! The scenraio viewer/editor.

use std::rc::Rc;

use yew::prelude::*;

use traffloat::def;
use traffloat::save;

pub mod building;
pub mod nav;

const SIDEBAR_WIDTH_PX: u32 = 200;
const SIDEBAR_PADDING_PX: u32 = 10;
const MAIN_WIDTH_PX: u32 = 750;

/// Displays an editor for ducts in an edge.
pub struct Comp {
    file: Rc<save::SaveFile>,
    link: ComponentLink<Self>,
    state: State,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        let file = match save::parse(&props.buf) {
            Ok(file) => Rc::new(file),
            Err(err) => {
                props
                    .close_hook
                    .emit(Some(format!("Error reading save file: {}", err)));
                return Self {
                    file: Default::default(), // this value shouldn't be used anyway.
                    link,
                    state: State::default(),
                };
            }
        };

        Self {
            file,
            link,
            state: State::default(),
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::EditorHome => {
                self.state.switch = Switch::Home;
                true
            }
            Msg::ChooseBuilding(id) => {
                self.state.switch = Switch::Building(id);
                true
            }
        }
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        unimplemented!("I don't know when editor::Comp would be re-rendered")
    }

    fn view(&self) -> Html {
        html! {
            <>
                <nav::Comp
                    file=Rc::clone(&self.file)
                    editor_home=self.link.callback(|()| Msg::EditorHome)
                    choose_building=self.link.callback(Msg::ChooseBuilding)
                    />
                <main style=format!("
                    margin-left: {}px;
                    border-left: 1px solid;
                    padding: 5px 10px;
                    height: 100vh;
                    font-family: 'Helvetica', 'Arial', sans-serif;
                ", SIDEBAR_WIDTH_PX + SIDEBAR_PADDING_PX)>
                    <div style=format!("
                        margin-left: auto;
                        margin-right: auto;
                        max-width: {}px;
                        overflow: auto;
                    ", MAIN_WIDTH_PX)>
                        { self.switch() }
                    </div>
                </main>
            </>
        }
    }
}

impl Comp {
    fn switch(&self) -> Html {
        match self.state.switch {
            Switch::Home => html! {
                <p>
                    { "Use buttons in the navbar to view/edit details." }
                </p>
            },
            Switch::Building(building_id) => html! {
                <building::detail::Comp
                    file=Rc::clone(&self.file)
                    building_id=building_id
                    />
            },
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
}

impl Default for Switch {
    fn default() -> Self {
        Self::Home
    }
}

/// Events for [`Comp`].
pub enum Msg {
    /// Set the main body to home.
    EditorHome,
    /// Set the main body to a building.
    ChooseBuilding(def::building::TypeId),
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub buf: Rc<[u8]>,
    pub close_hook: Callback<Option<String>>,
}
