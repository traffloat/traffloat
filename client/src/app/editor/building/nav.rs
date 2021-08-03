//! Navbar buildings.

use std::rc::Rc;

use yew::prelude::*;

use traffloat::def::building;
use traffloat::save;

/// Displays a list of buildings.
pub struct Comp {
    props: Props,
    link: ComponentLink<Self>,
    open: bool,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            open: false,
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::Toggle(_) => {
                self.open = !self.open;
                true
            }
            Msg::ChooseBuilding(id) => {
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
                    style="cursor: pointer;"
                    onclick=self.link.callback(Msg::Toggle)
                >
                    <h3>{ "Buildings" }</h3>
                </div>
                { for self.open.then(|| html! {
                    <div>
                        { for self.props.file.def().building_cats().iter()
                                .map(|(category_id, category)| html! {
                            <div>
                                <h4>{ category.title() }</h4>
                                { for self.props.file.def().building().iter()
                                        .filter(|(_, building)| building.category() == category_id)
                                        .map(|(building_id, building)| html! {
                                    <div
                                        style="cursor: pointer;"
                                        onclick=self.link.callback({
                                            let building_id = building_id.clone();
                                            move |_| Msg::ChooseBuilding(building_id.clone())
                                        })
                                    >
                                        <p>{ building.name() }</p>
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
    ChooseBuilding(building::TypeId),
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The loaded tsv file.
    pub file: Rc<save::SaveFile>,
    /// Set the main body to a building.
    pub choose_building: Callback<building::TypeId>,
}
