//! The fixed tab of the color picker dialog.

use yew::prelude::*;

/// The fixed tab of the color picker dialog.
pub struct Comp {
    props:       Props,
    link:        ComponentLink<Self>,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        Self { props, link }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
        }
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <palette::Comp callback=self.link.callback(Msg::Confirm) />
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
}
