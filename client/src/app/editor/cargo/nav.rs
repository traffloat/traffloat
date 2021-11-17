//! Navbar cargo.

use std::rc::Rc;

use traffloat::def::cargo;
use traffloat::save;
use yew::prelude::*;

use crate::app::lang;

/// Displays a list of cargo.
pub struct Comp {
    props: Props,
    link:  ComponentLink<Self>,
    open:  bool,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self { Self { props, link, open: false } }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::Toggle(_) => {
                self.open = !self.open;
                true
            }
            Msg::ChooseCargo(event, id) => {
                event.prevent_default();
                self.props.choose_cargo.emit(id);
                false
            }
        }
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <div
                    style=style!("cursor": "pointer")
                    onclick=self.link.callback(Msg::Toggle)
                >
                    <h3>{ "Cargo" }</h3>
                </div>
                { for self.open.then(|| html! {
                    <div>
                        { for self.props.def.cargo_category().iter()
                                .map(|category| html! {
                            <div>
                                <h4><lang::Comp item=category.title() /></h4>
                                { for self.props.def.cargo().iter()
                                        .filter(|cargo| cargo.category() == category.id())
                                        .map(|cargo| html! {
                                    <div>
                                        <a
                                            href=format!("#/{}/rules/cargo/{}", &self.props.route_prefix, cargo.id_str().value())
                                            onclick=self.link.callback({
                                                let id = cargo.id();
                                                move |event| Msg::ChooseCargo(event, id)
                                            })
                                        >
                                            <lang::Comp item=cargo.name() />
                                        </a>
                                    </div>
                                })}
                            </div>
                        })}
                    </div>
                })}
            </div>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {
    /// Toggle the opening of this navbar component.
    Toggle(MouseEvent),
    /// The user chooses a cargo.
    ChooseCargo(MouseEvent, cargo::Id),
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The loaded scenario definition.
    pub def:          Rc<save::GameDefinition>,
    /// Set the main body to a cargo.
    pub choose_cargo: Callback<cargo::Id>,
    /// The prefix in the hash-route, e.g. `scenario/vanilla`)
    pub route_prefix: String,
}
