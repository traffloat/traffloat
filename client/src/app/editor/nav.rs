//! Editor navbar.

use std::rc::Rc;

use traffloat::def::{building, cargo};
use traffloat::save;
use yew::prelude::*;

/// Container for all nav items.
pub struct Comp {
    props: Props,
}
impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, _link: ComponentLink<Self>) -> Self { Self { props } }

    fn update(&mut self, msg: Msg) -> ShouldRender { match msg {} }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <nav style=style!(
                "overflow-x": "hidden",
                "overflow-y": "auto",
                "position": "fixed",
                "left": "0",
                "width": format!("{}px", super::SIDEBAR_WIDTH_PX),
                "height": "100vh",
                "padding": format!("5px {}px", super::SIDEBAR_PADDING_PX),
                "padding": "5px {padding}px",
                "font-family": "'Helvetica', 'Arial', sans-serif",
            )>
                <div
                    style=style!("cursor": "pointer")
                    onclick=self.props.editor_home.reform(|_| ())
                >
                    { "Game Rules" }
                </div>
                <super::building::nav::Comp
                    def=Rc::clone(&self.props.def)
                    choose_building=self.props.choose_building.clone()
                    route_prefix=self.props.route_prefix.clone()
                    />
                <super::cargo::nav::Comp
                    def=Rc::clone(&self.props.def)
                    choose_cargo=self.props.choose_cargo.clone()
                    route_prefix=self.props.route_prefix.clone()
                    />
            </nav>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The loaded scenario definition.
    pub def:             Rc<save::GameDefinition>,
    /// Set the main body to editor home.
    pub editor_home:     Callback<()>,
    /// Set the main body to a building.
    pub choose_building: Callback<building::Id>,
    /// Set the main body to a cargo.
    pub choose_cargo:    Callback<cargo::Id>,
    /// The prefix in the hash-route, e.g. `scenario/vanilla`)
    pub route_prefix:    String,
}
