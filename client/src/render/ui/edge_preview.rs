//! Renders edge info preview.

use legion::world::SubWorld;
use legion::{Entity, EntityStore};
use yew::prelude::*;

use super::{duct_editor, Update, UpdaterRef};
use crate::input;
use traffloat::edge;
use traffloat::node;

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
        match msg {
            Msg::EditDucts(_) => {
                self.props.edit_duct.emit(Some(duct_editor::Args {
                    entity: self.props.args.entity,
                }));
                true
            }
        }
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
                <button onclick=self.link.callback(Msg::EditDucts)>{ "Edit" }</button>
            </div>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {
    /// Open the duct editor.
    EditDucts(MouseEvent),
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The yew-independent properties.
    pub args: Args,
    /// Callback to start duct editor
    pub edit_duct: Callback<Option<duct_editor::Args>>,
}

/// Yew-independent properties.
#[derive(Clone)]
pub struct Args {
    /// Entity ID of the edge.
    pub entity: Entity,
}

#[codegen::system]
#[read_component(node::Name)]
#[read_component(edge::Id)]
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
        if let Ok(_edge) = entity_entry.get_component::<edge::Id>() {
            Some(Args { entity })
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
