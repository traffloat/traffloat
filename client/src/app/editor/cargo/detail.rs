//! Cargo list.

use std::rc::Rc;

use traffloat::def::cargo;
use traffloat::save::GameDefinition;
use yew::prelude::*;

/// Displays a list of cargo.
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
        let def = &self.props.def;
        let cargo =
            def.cargo().get(&self.props.cargo_id).expect("Route references undefined cargo");

        html! {
            <h1>{ cargo.name() }</h1>
            // TODO
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The loaded scenario definition.
    pub def:      Rc<GameDefinition>,
    /// The type ID of the active cargo.
    pub cargo_id: cargo::Id,
}
