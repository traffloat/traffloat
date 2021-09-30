//! Scenario chooser.

use std::convert::TryInto;

use yew::prelude::*;

use crate::app::route::Route;
use crate::app::scenarios;

/// Displays a form for choosing a scenario.
pub struct Comp {
    props:  Props,
    link:   ComponentLink<Self>,
    choice: usize,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self {
        let choice = match &props.intent_route {
            Some(Route::Scenario { name, .. }) => {
                match scenarios::OPTIONS.iter().enumerate().find(|(_, def)| def.id == name) {
                    Some((ord, def)) => {
                        props.choose_scenario.emit(super::ChooseScenario {
                            scenario: Some(super::Scenario::Url(def.path)),
                            name:     Some(def.id.into()),
                            explicit: false,
                        });
                        ord
                    }
                    None => 0,
                }
            }
            Some(Route::Custom { .. }) => {
                props.choose_scenario.emit(super::ChooseScenario {
                    scenario: None,
                    name:     None,
                    explicit: false,
                });
                scenarios::OPTIONS.len()
            }
            _ => {
                props.choose_scenario.emit(super::ChooseScenario {
                    scenario: Some(super::Scenario::Url(
                        scenarios::OPTIONS.get(0).expect("scenarios::OPTIONS is empty").path,
                    )),
                    name:     Some("vanilla".into()),
                    explicit: false,
                });
                0
            }
        };

        Self { props, link, choice }
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
                let def = scenarios::OPTIONS.get(index);
                let scenario = def.as_ref().map(|def| super::Scenario::Url(def.path));
                let name = def.as_ref().map(|def| def.id.into());
                self.props.choose_scenario.emit(super::ChooseScenario {
                    scenario,
                    name,
                    explicit: true,
                });
                true
            }
            Msg::ChooseFile(cd) => {
                let file = match cd {
                    ChangeData::Files(files) => files.get(0),
                    _ => unreachable!(),
                };
                self.props.choose_scenario.emit(super::ChooseScenario {
                    scenario: file.map(super::Scenario::File),
                    name:     None,
                    explicit: true,
                });
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
                <select onchange=self.link.callback(Msg::ChooseScenario)>
                    { for scenarios::OPTIONS.iter().enumerate().map(|(index, def)| html! {
                        <option selected=index == self.choice>
                            { def.name }
                        </option>
                    })}
                    <option>{ "Open\u{2026}" }</option>
                </select>


                { for (self.choice == scenarios::OPTIONS.len()).then(|| html! {
                    <input
                        type="file"
                        onchange=self.link.callback(Msg::ChooseFile)
                        />
                })}
            </div>
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
    pub choose_scenario: Callback<super::ChooseScenario>,
    /// The intended route to navigate to.
    pub intent_route:    Option<Route>,
}
