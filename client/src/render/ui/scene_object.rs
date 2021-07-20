//! Renders scene object info.

use legion::world::SubWorld;
use legion::EntityStore;
use yew::prelude::*;

use super::{Update, UpdaterRef};
use crate::input;
use traffloat::graph;

/// Displays basic info about a scene object.
pub struct Comp {
    props: Props,
    link: ComponentLink<Self>,
}

impl Component for Comp {
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
        let style = "
            position: absolute;
            bottom: 0;
            left: 0;
            width: 5em; height: 5em;
            color: black;
            pointer-events: auto;
            background-color: white;
            font-size: large;
        ";
        match &self.props.info {
            Info::Node { node_name } => html! {
                <div style=style>
                    <p>{ node_name }</p>
                </div>
            },
            Info::Edge {} => html! {
                <div style=style>
                    <p>{ "Corridor" }</p>
                </div>
            },
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// Node or edge info.
    pub info: Info,
}

/// Node or edge info.
#[derive(Clone)]
pub enum Info {
    /// Info for a node.
    Node {
        /// Name of the targeted node.
        node_name: String,
    },
    /// Info for an edge.
    Edge {},
}

#[codegen::system]
#[read_component(graph::NodeName)]
#[read_component(graph::EdgeId)]
#[thread_local]
fn draw(
    #[resource] hover_target: &input::mouse::HoverTarget,
    #[resource] focus_target: &input::FocusTarget,
    world: &mut SubWorld,
    #[resource] updater_ref: &UpdaterRef,
) {
    let info = if let Some(entity) = focus_target.entity().or_else(|| hover_target.entity()) {
        let entity_entry = world
            .entry_ref(entity)
            .expect("Target entity does not exist"); // TODO what if user is hovering over node while deleting it?
        if let Ok(node_name) = entity_entry.get_component::<graph::NodeName>() {
            log::debug!("Node {:?}", node_name);
            Some(Props {
                info: Info::Node {
                    node_name: node_name.name().to_string(),
                },
            })
        } else if let Ok(edge) = entity_entry.get_component::<graph::EdgeId>() {
            log::debug!("Edge {:?}", edge);
            Some(Props {
                info: Info::Edge {},
            })
        } else {
            log::warn!("Target entity has unknown type");
            None
        }
    } else {
        None
    };

    updater_ref.call(Update::SetSceneObject(info));
}

/// Sets up legion ECS for scene object info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup)
}
