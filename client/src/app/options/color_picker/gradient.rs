//! The gradient tab of the color picker dialog.

use std::borrow::Cow;

use yew::prelude::*;

/// The gradient tab of the color picker dialog.
pub struct Comp {
    props: Props,
    link:  ComponentLink<Self>,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self { Self { props, link } }

    fn update(&mut self, msg: Msg) -> ShouldRender { match msg {} }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {}
    }
}

/// Events for [`Comp`].
pub enum Msg {}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {}
