//! Editor navbar.

use std::rc::Rc;

use yew::prelude::*;

use traffloat::def::building;
use traffloat::def::cargo;
use traffloat::save;

/// Container for all nav items.
pub struct Comp {
    props: Props,
}
impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {}
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <nav style=format!("
                overflow-x: hidden;
                overflow-y: auto;
                position: fixed;
                left: 0;
                width: {width}px;
                height: 100vh;
                padding: 5px {padding}px;
                font-family: 'Helvetica', 'Arial', sans-serif;
            ", width=super::SIDEBAR_WIDTH_PX, padding=super::SIDEBAR_PADDING_PX)>
                <div
                    style="cursor: pointer;"
                    onclick=self.props.editor_home.reform(|_| ())
                >
                    { "Game Rules" }
                </div>
                <super::building::nav::Comp
                    file=Rc::clone(&self.props.file)
                    choose_building=self.props.choose_building.clone()
                    />
                <super::cargo::nav::Comp
                    file=Rc::clone(&self.props.file)
                    choose_cargo=self.props.choose_cargo.clone()
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
    /// The loaded tsv file.
    pub file: Rc<save::SaveFile>,
    /// Set the main body to editor home.
    pub editor_home: Callback<()>,
    /// Set the main body to a building.
    pub choose_building: Callback<building::TypeId>,
    /// Set the main body to a cargo.
    pub choose_cargo: Callback<cargo::TypeId>,
}
