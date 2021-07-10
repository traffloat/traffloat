//! Renders node info.

use yew::prelude::*;

/// Displays basic info about a node.
pub struct NodeInfo {
    props: Props,
    link: ComponentLink<Self>,
}

impl Component for NodeInfo {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        Self { props, link }
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
            <div style="
                position: absolute;
                bottom: 0;
                left: 0;
                width: 5em; height: 5em;
                color: black;
                pointer-events: auto;
                background-color: white;
                font-size: large;
            ">
                <p>{ &self.props.node_name }</p>
            </div>
        }
    }
}

/// Events for [`NodeInfo`].
pub enum Msg {}

/// Yew properties for [`NodeInfo`].
#[derive(Clone, Properties)]
pub struct Props {
    /// Name of the targeted node.
    pub node_name: String,
}
