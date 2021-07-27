//! Renders duct editor.

use legion::Entity;
use yew::prelude::*;

/// Displays an editor for ducts in an edge.
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
        if props.args.entity == self.props.args.entity {
            return false;
        }

        true
    }

    fn view(&self) -> Html {
        let style = "
            width: 100vw;
            height: 100vh;
            pointer-events: fill;
        ";
        html! {
            <svg viewBox="0 0 100 100" style=style>
                <circle cx="50" cy="50" r="40" fill="grey" />
                <circle style="cursor: pointer;" cx="30" cy="60" r="5" fill="red" />
                <circle style="cursor: pointer;" cx="70" cy="70" r="5" fill="green" />
                <circle style="cursor: pointer;" cx="80" cy="50" r="3" fill="blue" />
            </svg>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The yew-independent properties.
    pub args: Args,
}

/// Yew-independent properties.
#[derive(Clone)]
pub struct Args {
    /// The entity to edit.
    pub entity: Entity,
}

/// Sets up legion ECS for duct editor rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup
}
