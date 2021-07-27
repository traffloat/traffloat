//! Renders duct editor.

use std::cell::RefCell;
use std::rc::Rc;

use legion::{Entity, EntityStore};
use yew::prelude::*;

use traffloat::edge;

pub mod duct;

/// Displays an editor for ducts in an edge.
pub struct Comp {
    props: Props,
    link: ComponentLink<Self>,
    state: Option<State>,
}

impl Comp {
    /// Update the cached fields
    pub fn update_props(&mut self) {
        let legion = self.props.legion.borrow();

        self.state = None;

        if let Ok(entry) = legion.world.entry_ref(self.props.args.entity) {
            self.state = match (
                entry.get_component::<edge::Size>(),
                entry.get_component::<edge::Design>(),
            ) {
                (Ok(size), Ok(design)) => Some(State {
                    size: size.radius(),
                    ducts: design
                        .ducts()
                        .iter()
                        .enumerate()
                        .map(|(index, duct)| Circle {
                            center: duct.center(),
                            radius: duct.radius(),
                            ty: duct.ty(),
                            original_index: Some(index),
                        })
                        .collect(),
                }),
                _ => None,
            }
        }
    }
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        let mut ret = Self {
            props,
            link,
            state: None,
        };
        ret.update_props();
        ret
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::DuctMouseDown((_index, _event)) => {
                // TODO
                true
            }
            Msg::DuctMouseUp((_index, _event)) => {
                // TODO
                true
            }
        }
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        if props.args.entity == self.props.args.entity {
            return false;
        }

        self.update_props();

        true
    }

    fn view(&self) -> Html {
        let style = "
            width: 100vw;
            height: 100vh;
            pointer-events: fill;
        ";
        if let Some(state) = self.state.as_ref() {
            const CANVAS_PADDING: f64 = 0.2;
            let origin = state.size * (1. + CANVAS_PADDING); // origin of the editor
            html! {
                <svg viewBox=format!("0 0 {0} {0}", state.size * (1. + CANVAS_PADDING) * 2.) style=style>
                    <circle
                        cx=origin.to_string()
                        cy=origin.to_string()
                        r=state.size.to_string()
                        fill="grey"
                        />
                    { for state.ducts.iter().enumerate().map(|(index, duct)| html! {
                        <duct::Comp
                            origin=origin
                            radius=duct.radius
                            center=duct.center
                            ty=duct.ty

                            mouse_down=self.link.callback(move |event| Msg::DuctMouseDown((index, event)))
                            mouse_up=self.link.callback(move |event| Msg::DuctMouseUp((index, event)))

                            index=index
                            />
                    }) }
                </svg>
            }
        } else {
            html! { <></> }
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {
    /// When the user presses down a duct.
    DuctMouseDown((usize, MouseEvent)),
    /// When the user releases mouse on a duct.
    DuctMouseUp((usize, MouseEvent)),
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The yew-independent properties.
    pub args: Args,
    /// The legion setup.
    pub legion: Rc<RefCell<traffloat::Legion>>,
}

/// Yew-independent properties.
#[derive(Clone)]
pub struct Args {
    /// The entity to edit.
    pub entity: Entity,
}

/// The temporary state of edges in the duct
struct State {
    size: f64,
    ducts: Vec<Circle>,
}

/// A temporary circle in the edge.
#[derive(Debug, Clone, Copy)]
pub struct Circle {
    /// Position of the cicle center
    pub center: edge::CrossSectionPosition,
    /// Radius of the circle
    pub radius: f64,
    /// Type of the circle.
    pub ty: edge::DuctType,
    /// The index of this circle in [`edge::Design::ducts`] if it is not newly drawn.
    pub original_index: Option<usize>,
}
