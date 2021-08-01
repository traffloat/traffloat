//! Navbar cargo.

use std::rc::Rc;

use yew::prelude::*;

use traffloat::def::cargo;
use traffloat::save;

/// Displays a list of cargo.
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
            Msg::ChooseCargo(id) => {
                self.props.choose_cargo.emit(cargo::TypeId(id));
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
                    <h3>{ "Cargo" }</h3>
                </div>
                { for self.open.then(|| html! {
                    <div>
                        { for self.props.file.def().cargo_cats().iter().enumerate()
                                .map(|(category_id, category)| html! {
                            <div>
                                <h4>{ category.title() }</h4>
                                { for self.props.file.def().cargo().iter()
                                        .enumerate()
                                        .filter(|(_, cargo)| cargo.category().0 == category_id)
                                        .map(|(cargo_id, cargo)| html! {
                                    <div
                                        style="cursor: pointer;"
                                        onclick=self.link.callback(move |_| Msg::ChooseCargo(cargo_id))
                                    >
                                        <p>{ cargo.name() }</p>
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
    ChooseCargo(usize),
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The loaded tsv file.
    pub file: Rc<save::SaveFile>,
    /// Set the main body to a cargo.
    pub choose_cargo: Callback<cargo::TypeId>,
}