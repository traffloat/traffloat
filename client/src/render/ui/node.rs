//! Renders node info.

use legion::world::SubWorld;
use legion::EntityStore;
use yew::prelude::*;

use super::{Update, UpdaterRef};
use crate::input;
use traffloat::graph;

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

#[codegen::system]
#[read_component(graph::NodeName)]
#[thread_local]
fn draw(
    #[resource] hover_target: &input::mouse::HoverTarget,
    #[resource] focus_target: &input::FocusTarget,
    world: &mut SubWorld,
    #[resource] updater_ref: &UpdaterRef,
) {
    let info = if let Some(entity) = focus_target.entity().or_else(|| hover_target.entity()) {
        let node_name = world
            .entry_ref(entity)
            .expect("Target entity does not exist") // TODO what if user is hovering over node while deleting it?
            .into_component::<graph::NodeName>()
            .expect("Component NodeName does not exist in target entity");
        Some(Props {
            node_name: node_name.name().to_string(),
        })
    } else {
        None
    };

    updater_ref.call(Update::SetNodeInfo(info));
}

/// Sets up legion ECS for node info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup)
}
