//! Building list.

use std::rc::Rc;

use itertools::Itertools;
use yew::prelude::*;

use traffloat::def::feature::{reaction, Feature};
use traffloat::def::{building, catalyst, GameDefinition};
use traffloat::save::SaveFile;

/// Displays a list of buildings.
pub struct Comp {
    props: Props,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {}
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let def = self.props.file.def();
        let building = def
            .building()
            .get(&self.props.building_id)
            .expect("Route references undefined building");

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
                <h1>{ building.name() }</h1>
                <div style="
                    float: right;
                ">
                    <table>
                        <tbody>
                            { table_entry("Hitpoints", building.hitpoint()) }
                            { table_entry("Cargo capacity", building.storage().cargo()) }
                            { table_entry(
                                "Liquid capacity",
                                building
                                    .storage()
                                    .liquid()
                                    .iter()
                                    .map(|volume| volume.to_string())
                                    .join(" + ")
                            ) }
                            { table_entry("Gas capacity", building.storage().gas()) }
                        </tbody>
                    </table>
                </div>
                <p style="font-style: italic;">{ building.summary() }</p>
                <p>{ building.description() }</p>

                <h2>{ "Mechanisms" }</h2>
                { for building.features().iter().map(|feature| render_feature(feature, def)) }
            </>
        }
    }
}

fn render_feature(feature: &Feature, def: &GameDefinition) -> Html {
    match feature {
        Feature::Core => html! {
            <div>
                <h3>{ "Core" }</h3>
                <p>{ "This is a core building. The game is lost if all core buildings are destroyed." }</p>
            </div>
        },
        Feature::ProvidesHousing(capacity) => html! {
            <div>
                <h3>{ format_args!("Housing ({} capacity)", capacity) }</h3>
                <p>
                    { format_args!("This building provides {} housing capacity.", capacity) }
                    { "Inhabitants assigned to this building will be affected by " }
                    { "the skill-related mechanisms of this building, such as food, " }
                    { "even if they are not currently inside the building." }
                </p>
            </div>
        },
        Feature::Reaction(reaction) => render_reaction(reaction, def),
        Feature::RailPump(spec) => html! {
            <div>
                <h3>{ "Rail terminal" }</h3>
                <p>
                    { format_args!("Vehicles in adjacent corridors received a boost of {}.", spec.force()) }
                </p>
            </div>
        },
        Feature::LiquidPump(spec) => html! {
            <div>
                <h3>{ "Liquid pump" }</h3>
                <p>
                    { format_args!("Pipes in adjacent corridors receive a boost of {}.", spec.force()) }
                </p>
            </div>
        },
        Feature::GasPump(spec) => html! {
            <div>
                <h3>{ "Gas fan" }</h3>
                <p>
                    { "Fans can be installed on adjacent corridors to speed up gas diffusion. " }
                    { format_args!("Each fan provides up to {} of pumping force.", spec.force()) }
                </p>
            </div>
        },
        Feature::SecureEntry(policy) => {
            html! {
                <div>
                    <h3>{ "Entry security" }</h3>
                    <p>
                        { "TODO" }
                    </p>
                </div>
            }
        }
        Feature::SecureExit(policy) => {
            html! {
                <div>
                    <h3>{ "Exit security" }</h3>
                    <p>
                        { "TODO" }
                    </p>
                </div>
            }
        }
    }
}

fn render_reaction(reaction: &reaction::Reaction, def: &GameDefinition) -> Html {
    html! {
        <div>
            <p>
                { format_args!(
                    "The rate of reaction {} be configured.",
                    if reaction.policy().configurable() { "can" } else { "cannot" }
                ) }
            </p>
            // TODO document FlowPolicy

            <table>
                <tbody style="text-align: center;">
                    <tr>
                        <th style="width: 20%;" rowspan="2">{ "Condition" }</th>
                        <th style="width: 20%;" rowspan="2">{ "Optimal range" }</th>
                        <th colspan="3">{ "Rate multiplier" }</th>
                    </tr>
                    <tr>
                        <th style="width: 20%;">{ "Below optimal range" }</th>
                        <th style="width: 20%;"> { "Within optimal range" } </th>
                        <th style="width: 20%;">{ "Above optimal range" }</th>
                    </tr>

                    { for reaction.catalysts().iter().map(|catalyst| html! {
                        <tr>
                            { match catalyst.range() {
                                    catalyst::CatalystRange::Cargo { ty, levels } => html! {
                                        <>
                                            <td>{ def.cargo().get(ty).expect("Save references undefined cargo").name() }</td>
                                            <td>{ format_args!(
                                                "{} to {}",
                                                levels.start,
                                                levels.end,
                                            ) }</td>
                                        </>
                                    },
                                    catalyst::CatalystRange::Liquid { ty, levels } => html! {
                                        <>
                                            <td>{ def.liquid().get(ty).expect("Save references undefined liquid").name() }</td>
                                            <td>{ format_args!(
                                                "{} to {}",
                                                levels.start,
                                                levels.end,
                                            ) }</td>
                                        </>
                                    },
                                    catalyst::CatalystRange::Gas { ty, levels } => html! {
                                        <>
                                            <td>{ def.gas().get(ty).expect("Save references undefined gas").name() }</td>
                                            <td>{ format_args!(
                                                "{} to {}",
                                                levels.start,
                                                levels.end,
                                            ) }</td>
                                        </>
                                    },
                                    catalyst::CatalystRange::Electricity { levels } => html! {
                                        <>
                                            <td>{ "Electricity" }</td>
                                            <td>{ format_args!(
                                                "{} to {}",
                                                levels.start,
                                                levels.end,
                                            ) }</td>
                                        </>
                                    },
                                    catalyst::CatalystRange::Light { levels } => html! {
                                        <>
                                            <td>{ "Sunlight" }</td>
                                            <td>{ format_args!(
                                                "{} to {}",
                                                levels.start,
                                                levels.end,
                                            ) }</td>
                                        </>
                                    },
                                    catalyst::CatalystRange::Skill { ty, levels } => html! {
                                        <>
                                            <td>{ format_args!(
                                                "Operator with {}",
                                                def.skill().get(ty).expect("Save references undefined skill").name(),
                                            ) }</td>
                                            <td>{ format_args!(
                                                "{} to {}",
                                                levels.start,
                                                levels.end,
                                            ) }</td>
                                        </>
                                    },
                            } }
                            <td>{ format_args!("{}\u{d7}", catalyst.multipliers().underflow()) }</td>
                            <td>{ format_args!(
                                "{}\u{d7} to {}\u{d7}",
                                catalyst.multipliers().min(),
                                catalyst.multipliers().max(),
                            ) }</td>
                            <td>{ format_args!("{}\u{d7}", catalyst.multipliers().overflow()) }</td>
                        </tr>
                    }) }
                </tbody>
            </table>

            // TODO render inputs/outputs
        </div>
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
