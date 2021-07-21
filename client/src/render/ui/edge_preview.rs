//! Renders edge info preview.

use legion::world::SubWorld;
use legion::{Entity, EntityStore};
use yew::prelude::*;

use super::{Update, UpdaterRef};
use crate::input;
use traffloat::graph;

/// Displays basic info about an edge at a corner of the screen.
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
        html! {
            <div style=style>
                <p>{ "Corridor" }</p>
            </div>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// Entity ID of the edge.
    pub entity: Entity,
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
        if let Ok(_edge) = entity_entry.get_component::<graph::EdgeId>() {
            Some(Props { entity })
        } else {
            None
        }
    } else {
        None
    };

    updater_ref.call(Update::SetEdgePreview(info));
}

/// Sets up legion ECS for edge info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup)
}
