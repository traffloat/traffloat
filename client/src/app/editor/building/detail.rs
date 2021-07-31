//! Building list.

use std::rc::Rc;

use yew::prelude::*;

use traffloat::def::building;
use traffloat::save::SaveFile;

/// Displays a list of buildings.
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
        let building = self.props.file.def().get_building(self.props.building_id);

        html! {
            <>
                <h2>{ building.name() }</h2>
                <p style="font-style: italic;">{ building.summary() }</p>
                <p>{ building.description() }</p>
                <p>
                    { "Hitpoints: " }
                    { building.hitpoint() }
                </p>
                <p>
                    { "Cargo capacity: " }
                    { building.storage().cargo() }
                </p>
                <p>
                    { "Liquid capacity: " }
                    { building.storage().liquid() }
                </p>
                <p>
                    { "Gas capacity: " }
                    { building.storage().gas() }
                </p>
            </>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The loaded tsv file.
    pub file: Rc<SaveFile>,
    /// The type ID of the active building.
    pub building_id: building::TypeId,
}
