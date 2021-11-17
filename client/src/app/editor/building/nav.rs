//! Navbar buildings.

use std::rc::Rc;

use traffloat::def::building;
use traffloat::save::GameDefinition;
use yew::prelude::*;

use crate::app::lang;

/// Displays a list of buildings.
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
            Msg::ChooseBuilding(event, id) => {
                event.prevent_default();
                self.props.choose_building.emit(id);
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
                    <h3>{ "Buildings" }</h3>
                </div>
                { for self.open.then(|| html! {
                    <div>
                        { for self.props.def.building_category().iter()
                                .map(|category| html! {
                            <div>
                                <h4>
                                    <lang::Comp item=category.title() />
                                </h4>
                                { for self.props.def.building().iter()
                                        .filter(|building| building.category() == category.id())
                                        .map(|building| html! {
                                    <div>
                                        <a
                                            href=format!("#/{}/rules/building/{}", &self.props.route_prefix, building.id_str().value())
                                            onclick=self.link.callback({
                                                let id = building.id();
                                                move |event| Msg::ChooseBuilding(event, id)
                                            })
                                        >
                                            <lang::Comp item=building.name() />
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
    /// The user chooses a building.
    ChooseBuilding(MouseEvent, building::Id),
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The loaded scenario definition.
    pub def:             Rc<GameDefinition>,
    /// Set the main body to a building.
    pub choose_building: Callback<building::Id>,
    /// The prefix in the hash-route, e.g. `scenario/vanilla`)
    pub route_prefix:    String,
}
