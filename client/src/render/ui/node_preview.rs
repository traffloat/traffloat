//! Renders node info preview.

use legion::world::SubWorld;
use legion::{Entity, EntityStore};
use traffloat::clock::Clock;
use traffloat::config::Scalar;
use traffloat::def::CustomizableName;
use traffloat::{cargo, edge, gas, liquid, save, units};
use traffloat_def::lang;
use yew::prelude::*;

use super::{Update, UpdaterRef};
use crate::{app, input, ContextPath};

/// Displays basic info about a node at a corner of the screen.
pub struct Comp {
    props: Props,
    link:  ComponentLink<Self>,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Props, link: ComponentLink<Self>) -> Self { Self { props, link } }

    fn update(&mut self, msg: Msg) -> ShouldRender {
        match msg {
            Msg::HelpButton(_) => {
                // TODO display node details
                true
            }
        }
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        use traffloat::units::RoundedUnit;

        fn storage_display(
            size: impl RoundedUnit + Into<Html>,
            _name: &lang::Item,
            icon: &str,
        ) -> Html {
            html! {
                <tr>
                    <td>{ size.round(2) }</td>
                    <td>
                        <span> // TODO render translated name as title
                            <img
                                src=icon.to_string()
                                width=24
                                height=24
                                />
                        </span>
                    </td>
                </tr>
            }
        }

        style! { static ROW_STYLE =
            "margin": "0.5em 0",
        }
        let row_style = &*ROW_STYLE;

        html! {
            <div style=style!(
                "position": "absolute",
                "bottom": "0",
                "left": "0",
                "width": "15em", "height": "10em",
                "color": "black",
                "pointer-events": "auto",
                "background-color": "white",
                "font-size": "large",
                "padding": "1em 0.2em",
                "overflow-y": "auto",
            )>
                <p
                    onclick=self.link.callback(Msg::HelpButton)
                    style=style!(
                        ..ROW_STYLE,
                        "cursor": "help",
                    )
                >
                    <app::lang::Comp item=&self.props.args.node_name />
                </p>
                <p style=row_style>
                    { self.props.args.hitpoint }
                </p>
                <table style=row_style>
                    { for self.props.args.cargo.iter().map(|entry| storage_display(entry.size, &entry.name, &entry.icon)) }
                    { for self.props.args.liquid.iter().map(|entry| storage_display(entry.size, &entry.name, &entry.icon)) }
                    { for self.props.args.gas.iter().map(|entry| storage_display(entry.size, &entry.name, &entry.icon)) }
                </table>
            </div>
        }
    }
}

/// Events for [`Comp`].
pub enum Msg {
    /// The user clicks the help button.
    HelpButton(MouseEvent),
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The yew-independent properties.
    pub args: Args,
}

/// Yew-independent properties.
#[derive(Clone)]
pub struct Args {
    /// Entity ID of the node.
    pub entity:    Entity,
    /// Name of the targeted node.
    pub node_name: CustomizableName,
    /// Hitpoint value of the targeted node.
    pub hitpoint:  units::Portion<units::Hitpoint>,
    /// Cargo stored in the targeted node.
    pub cargo:     Vec<StorageEntry<units::CargoSize>>,
    /// Liquids stored in the targeted node.
    pub liquid:    Vec<StorageEntry<units::LiquidVolume>>,
    /// Gases stored in the targeted node.
    pub gas:       Vec<StorageEntry<units::GasVolume>>,
}

/// Preview information for a storage.
#[derive(Clone)]
pub struct StorageEntry<S: units::Unit> {
    size: S,
    name: lang::Item,
    icon: String,
}

#[codegen::system(Visualize)]
#[read_component(CustomizableName)]
#[read_component(edge::Id)]
#[read_component(units::Portion<units::Hitpoint>)]
#[read_component(cargo::StorageList)]
#[read_component(cargo::StorageSize)]
#[read_component(cargo::NextStorageSize)]
#[read_component(liquid::StorageList)]
#[read_component(liquid::Storage)]
#[read_component(liquid::StorageSize)]
#[read_component(liquid::NextStorageSize)]
#[read_component(gas::StorageList)]
#[read_component(gas::StorageSize)]
#[read_component(gas::NextStorageSize)]
#[thread_local]
fn draw(
    #[resource] hover_target: &input::mouse::HoverTarget,
    #[resource] focus_target: &input::FocusTarget,
    world: &mut SubWorld,
    #[resource] updater_ref: &UpdaterRef,
    #[resource] clock: &Clock,
    #[resource(no_init)] def: &save::GameDefinition,
    #[resource(no_init)] context_path: &ContextPath,
    #[resource] config: &Scalar,
) {
    let info = if let Some(entity) = focus_target.entity().or_else(|| hover_target.entity()) {
        let entity_entry = world.entry_ref(entity).expect("Target entity does not exist"); // TODO what if user is hovering over node while deleting it?
        if let (Ok(node_name), Ok(&hitpoint), Ok(cargo_list), Ok(liquid_list), Ok(gas_list)) = (
            entity_entry.get_component::<CustomizableName>(),
            entity_entry.get_component::<units::Portion<units::Hitpoint>>(),
            entity_entry.get_component::<cargo::StorageList>(),
            entity_entry.get_component::<liquid::StorageList>(),
            entity_entry.get_component::<gas::StorageList>(),
        ) {
            macro_rules! read_storage {
                ($list:expr, $mod:ident) => {
                    $list
                        .storages()
                        .iter()
                        .map(|&(id, entity)| {
                            let storage_entry =
                                world.entry_ref(entity).expect("Storage entity does not exist");
                            let size = storage_entry
                                .get_component::<$mod::StorageSize>()
                                .expect("Storage has no size");
                            let next_size = storage_entry
                                .get_component::<$mod::NextStorageSize>()
                                .expect("Storage has no next size");
                            let lerp_size = $mod::lerp(size, next_size, clock.now());
                            let item = &def[id];
                            let name = item.name();
                            let icon = format!(
                                "{}/{}",
                                context_path.as_ref(),
                                item.texture().spritesheet_id().value()
                            );
                            StorageEntry { size: lerp_size, name: name.clone(), icon }
                        })
                        .collect()
                };
            }

            let cargo = read_storage!(cargo_list, cargo);
            let gas = read_storage!(gas_list, gas);

            let liquid = liquid_list
                .storages()
                .iter()
                .filter_map(|entity| {
                    let storage_entry =
                        world.entry_ref(*entity).expect("Storage entity does not exist");
                    let storage = storage_entry
                        .get_component::<liquid::Storage>()
                        .expect("Storage has no storage");
                    let size = storage_entry
                        .get_component::<liquid::StorageSize>()
                        .expect("Storage has no size");
                    let next_size = storage_entry
                        .get_component::<liquid::NextStorageSize>()
                        .expect("Storage has no next size");
                    let lerp_size = liquid::lerp(size, next_size, clock.now());

                    if lerp_size < config.negligible_volume {
                        return None;
                    }

                    let item = &def[storage.liquid()];
                    let name = item.name();
                    let icon = format!(
                        "{}/{}",
                        context_path.as_ref(),
                        item.texture().spritesheet_id().value()
                    );

                    Some(StorageEntry { size: lerp_size, name: name.clone(), icon })
                })
                .collect();

            Some(Args { entity, node_name: node_name.clone(), hitpoint, cargo, liquid, gas })
        } else {
            None
        }
    } else {
        None
    };

    updater_ref.call(Update::SetNodePreview(info));
}

/// Sets up legion ECS for node info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs { setup.uses(draw_setup) }
