//! The button to select tab in color picker.

use std::borrow::Cow;

use yew::prelude::*;

/// The button to select tab in color picker.
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
        html! {
            <button
                onclick=self.props.callback.reform(|_| ())
                style=style!(
                    "height": "2em",
                    "flex-grow": "1",
                )
            >
                { self.props.title }
            </button>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    pub title:    &'static str,
    pub callback: Callback<()>,
}
