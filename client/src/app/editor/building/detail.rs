//! Building list.

use std::rc::Rc;

use traffloat::def::feature::{reaction, Feature};
use traffloat::def::{building, catalyst};
use traffloat::save::GameDefinition;
use yew::prelude::*;

use crate::app::lang;

/// Displays a list of buildings.
pub struct Comp {
    props: Props,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, _link: ComponentLink<Self>) -> Self { Self { props } }

    fn update(&mut self, msg: Msg) -> ShouldRender { match msg {} }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        let def = &self.props.def;
        let building = &def[self.props.building_id];

        fn table_entry(name: impl Into<Html>, value: impl Into<Html>) -> Html {
            html! {
                <tr>
                    <td style=style!("width": "4em", "padding-right": "10px")>{ name }</td>
                    <td style=style!("width": "8em;")>{ value }</td>
                </tr>
            }
        }

        html! {
            <>
                <h1><lang::Comp item=building.name() /></h1>
                <div style=style!("float": "right")>
                    <table>
                        <tbody>
                            { table_entry("Hitpoints", building.hitpoint()) }
                            { table_entry("Cargo capacity", building.storage().cargo()) }
                            { for building.storage().liquid().iter().map(|storage| {
                                table_entry(html! {
                                    <>
                                        { "Liquid capacity (" }
                                        <lang::Comp item=storage.name() />
                                        { ")" }
                                    </>
                                }, storage.capacity())
                            }) }
                            { table_entry("Gas capacity", building.storage().gas()) }
                            { for building.storage().population().iter().map(|storage| {
                                table_entry(html! {
                                    <>
                                        { "Inhabitant capacity (" }
                                        <lang::Comp item=storage.name() />
                                        { ")" }
                                    </>
                                }, storage.capacity())
                            }) }
                        </tbody>
                    </table>
                </div>
                <p style=style!("font-style": "italic")>
                    <lang::Comp item=building.summary() />
                </p>
                <p>
                    <lang::Comp item=building.description() />
                </p>

                <h2>{ "Mechanisms" }</h2>
                { for building.features().iter().map(|feature| render_feature(building, feature, def)) }
            </>
        }
    }
}

fn render_feature(building: &building::Def, feature: &Feature, def: &GameDefinition) -> Html {
    match feature {
        Feature::Core => html! {
            <div>
                <h3>{ "Core" }</h3>
                <p>{ "This is a core building. The game is lost if all core buildings are destroyed." }</p>
            </div>
        },
        Feature::ProvidesHousing(housing) => {
            let storage = building
                .storage()
                .population()
                .get(housing.storage().index())
                .expect("Corrupted definition");

            html! {
                <div>
                    <h3>{ "Housing" }</h3>
                    <p>
                        { "The " }
                        <lang::Comp item=storage.name() />
                        { " facility provides housing to occupants. " }
                        { "Inhabitants assigned to this building will be affected by " }
                        { "the skill-related mechanisms of this building, such as food, " }
                        { "even if they are not currently inside the building." }
                    </p>
                </div>
            }
        }
        Feature::Reaction(reaction) => render_reaction(reaction, building, def),
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
        Feature::SecureEntry(policy) | Feature::SecureExit(policy) => {
            html! {
                <div>
                    <h3>{ match feature {
                        Feature::SecureEntry(_) => "Entry security",
                        Feature::SecureExit(_) => "Exit security",
                        _ => unreachable!(),
                    } }</h3>

                    <p>
                        { match feature {
                            Feature::SecureEntry(_) => format!("\
                                Unqualified inhabitants have {0}% probability to succeed breaking in.
                                Rejected inhabitants are immediately teleported back to their original building. \
                                The inhabitant will then attempt to find alternative paths to the destination. \
                                If there is no alternative path, the inhabitant will try to travel across the corridor again \
                                and get another {0}% chance to break in.", policy.breach_probability() * 100.),
                            Feature::SecureExit(_) => format!("\
                                Unqualified inhabitants have {0}% probability to succed breaking out.
                                Rejected inhabitants will stay immobile for one second \
                                and get another {0}% chance to break out.", policy.breach_probability() * 100.),
                            _ => unreachable!(),
                        } }
                    </p>

                    { for (!policy.catalysts().is_empty()).then(|| html! {
                        <>
                            <h4>{ "Boosts" }</h4>
                            { for policy.catalysts().iter().map(|catalyst| render_catalyst(catalyst, building, def)) }
                        </>
                    }) }
                </div>
            }
        }
    }
}

fn render_reaction(
    reaction: &reaction::Reaction,
    building: &building::Def,
    def: &GameDefinition,
) -> Html {
    html! {
        <div>
            <h3><lang::Comp item=reaction.title() /></h3>
            <p><lang::Comp item=reaction.description() /></p>
            <p>
                { format_args!(
                    "The rate of reaction {} be configured.",
                    if reaction.policy().configurable() { "can" } else { "cannot" }
                ) }
            </p>
            // TODO document FlowPolicy

            <table>
                <tbody style=style!("text-align": "center")>
                    <tr>
                        <th style=style!("width": "20%") rowspan="2">{ "Condition" }</th>
                        <th style=style!("width": "20%") rowspan="2">{ "Optimal range" }</th>
                        <th colspan="3">{ "Rate multiplier" }</th>
                    </tr>
                    <tr>
                        <th style=style!("width": "20%")>{ "Below optimal range" }</th>
                        <th style=style!("width": "20%")> { "Within optimal range" } </th>
                        <th style=style!("width": "20%")>{ "Above optimal range" }</th>
                    </tr>

                    { for reaction.catalysts().iter().map(|catalyst| html! {
                        <tr>
                        { render_catalyst(catalyst, building, def) }
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

fn render_catalyst(
    catalyst: &catalyst::Catalyst,
    building: &building::Def,
    def: &GameDefinition,
) -> Html {
    match catalyst.range() {
        catalyst::CatalystRange::Cargo { ty, levels } => html! {
            <>
                <td><lang::Comp item=def[*ty].name() /></td>
                <td>{ format_args!(
                    "{} to {}",
                    levels.start,
                    levels.end,
                ) }</td>
            </>
        },
        catalyst::CatalystRange::Liquid { ty, levels } => html! {
            <>
                <td><lang::Comp item=def[*ty].name() /></td>
                <td>{ format_args!(
                    "{} to {}",
                    levels.start,
                    levels.end,
                ) }</td>
            </>
        },
        catalyst::CatalystRange::Gas { ty, levels } => html! {
            <>
                <td><lang::Comp item=def[*ty].name() /></td>
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
        catalyst::CatalystRange::Skill { ty, levels, storage } => html! {
            <>
                <td>
                    { "Inhabitants (" }
                    {{
                        let storage = building.storage().population().get(storage.index()).expect("Corrupted definition");
                        html!(<lang::Comp item=storage.name() />)
                    }}
                    { ") with " }
                    <lang::Comp item=def[*ty].name() />
                </td>
                <td>{ format_args!(
                    "{} to {}",
                    levels.start,
                    levels.end,
                ) }</td>
            </>
        },
    }
}

/// Events for [`Comp`].
pub enum Msg {}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The loaded scenario definition.
    pub def:         Rc<GameDefinition>,
    /// The type ID of the active building.
    pub building_id: building::Id,
}
