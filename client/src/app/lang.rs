//! Displays a translatable span.
use traffloat_def::CustomizableName;
use yew::prelude::*;

/// Displays a translatable span.
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
            { format!("{:?}", &self.props.item) }
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    pub item: CustomizableName,
}
