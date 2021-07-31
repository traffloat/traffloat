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

        fn table_entry(name: impl Into<Html>, value: impl Into<Html>) -> Html {
            html! {
                <tr>
                    <td style="width: 4em; padding-right: 10px;">{ name }</td>
                    <td style="width: 8em;">{ value }</td>
                </tr>
            }
        }

        html! {
            <>
                <h2>{ building.name() }</h2>
                <p style="font-style: italic;">{ building.summary() }</p>
                <p>{ building.description() }</p>
                <div style="
                    float: right;
                ">
                    <table>
                        <tbody>
                            { table_entry("Hitpoints", building.hitpoint()) }
                            { table_entry("Cargo capacity", building.storage().cargo()) }
                            { table_entry("Liquid capacity", building.storage().liquid()) }
                            { table_entry("Gas capacity", building.storage().gas()) }
                        </tbody>
                    </table>
                </div>
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
