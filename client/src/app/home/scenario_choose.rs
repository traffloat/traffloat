//! Scenario chooser.

use std::convert::TryInto;

use yew::prelude::*;

pub const SCENARIO_OPTIONS: &[(&str, &str)] = &[("Vanilla", "vanilla.tsvt")];

/// Displays a form for choosing a scenario.
pub struct Comp {
    props: Props,
    link: ComponentLink<Self>,
    select_ref: NodeRef,
    choice: usize,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            select_ref: NodeRef::default(),
            choice: 0,
        }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::ChooseScenario(cd) => {
                let index = match cd {
                    ChangeData::Select(el) => el.selected_index(),
                    _ => unreachable!(),
                }
                .try_into()
                .expect("Index out of bounds");
                self.choice = index;
                self.props.choose_scenario.emit(
                    SCENARIO_OPTIONS
                        .get(index)
                        .map(|(_, url)| super::Scenario::Url(url)),
                );
                true
            }
            Msg::ChooseFile(cd) => {
                let file = match cd {
                    ChangeData::Files(files) => files.get(0),
                    _ => unreachable!(),
                };
                self.props
                    .choose_scenario
                    .emit(file.map(super::Scenario::File));
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
                <label>{ "Select scenario" }</label>
                <select ref=self.select_ref.clone() onchange=self.link.callback(Msg::ChooseScenario)>
                    { for SCENARIO_OPTIONS.iter().enumerate().map(|(index, (name, _url))| html! {
                        <option selected=index == self.choice>
                            { name }
                        </option>
                    })}
                    <option>{ "Open\u{2026}" }</option>
                </select>


                { for (self.choice == SCENARIO_OPTIONS.len()).then(|| html! {
                    <input
                        type="file"
                        onchange=self.link.callback(Msg::ChooseFile)
                        />
                })}
            </div>
        }
    }

    fn rendered(&mut self, first: bool) {
        if first {
            self.link
                .send_message(Msg::ChooseScenario(ChangeData::Select(
                    self.select_ref
                        .cast()
                        .expect("<select> is not HtmlSelectElement"),
                )))
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {
    /// The scenario chooser is changed.
    ChooseScenario(ChangeData),
    /// The custom scenario file is changed.
    ChooseFile(ChangeData),
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The callback for updating chosen scenario.
    pub choose_scenario: Callback<Option<super::Scenario>>,
}
