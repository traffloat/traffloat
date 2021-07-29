//! Renders node info preview.

use arcstr::ArcStr;
use legion::world::SubWorld;
use legion::{Entity, EntityStore};
use smallvec::SmallVec;
use yew::prelude::*;

use super::{Update, UpdaterRef};
use crate::input;
use traffloat::clock::Clock;
use traffloat::def::GameDefinition;
use traffloat::units;
use traffloat::{cargo, gas, liquid};
use traffloat::{edge, node};

/// Displays basic info about a node at a corner of the screen.
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
        match msg {
            Msg::HelpButton(_) => {
                // TODO display node details
                true
            }
        }
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        let changed = self.props.args != props.args;
        self.props = props;
        changed
    }

    fn view(&self) -> Html {
        let style = "
            position: absolute;
            bottom: 0;
            left: 0;
            width: 15em; height: 10em;
            color: black;
            pointer-events: auto;
            background-color: white;
            font-size: large;
        ";
        html! {
            <div style=style>
                <p
                    onclick=self.link.callback(Msg::HelpButton)
                    style="cursor: help;"
                >
                    { &self.props.args.node_name }
                </p>
                <p>
                    { self.props.args.hitpoint }
                </p>
                <p>
                    { for self.props.args.cargo.iter().map(|(size, name)| html! {
                        <>
                            { size }
                            { " " }
                            { name }
                            <br />
                        </>
                    }) }
                </p>
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
#[derive(Clone, PartialEq)]
pub struct Args {
    /// Entity ID of the node.
    pub entity: Entity,
    /// Name of the targeted node.
    pub node_name: ArcStr,
    /// Hitpoint value of the targeted node.
    pub hitpoint: units::Portion<units::Hitpoint>,
    /// Cargo stored in the targeted node.
    pub cargo: SmallVec<[(units::CargoSize, ArcStr); 4]>,
    /// Liquids stored in the targeted node.
    pub liquid: SmallVec<[(units::LiquidVolume, ArcStr); 4]>,
    /// Gases stored in the targeted node.
    pub gas: SmallVec<[(units::GasVolume, ArcStr); 4]>,
}

#[codegen::system]
#[read_component(node::Name)]
#[read_component(edge::Id)]
#[read_component(units::Portion<units::Hitpoint>)]
#[read_component(cargo::StorageList)]
#[read_component(cargo::StorageSize)]
#[read_component(cargo::NextStorageSize)]
#[read_component(liquid::StorageList)]
#[read_component(liquid::StorageSize)]
#[read_component(liquid::NextStorageSize)]
#[read_component(gas::StorageList)]
#[read_component(gas::StorageSize)]
#[read_component(gas::NextStorageSize)]
#[thread_local]
fn draw(
    #[resource(no_init)] def: &GameDefinition,
    #[resource] hover_target: &input::mouse::HoverTarget,
    #[resource] focus_target: &input::FocusTarget,
    world: &mut SubWorld,
    #[resource] updater_ref: &UpdaterRef,
    #[resource] clock: &Clock,
) {
    let info = if let Some(entity) = focus_target.entity().or_else(|| hover_target.entity()) {
        let entity_entry = world
            .entry_ref(entity)
            .expect("Target entity does not exist"); // TODO what if user is hovering over node while deleting it?
        if let (Ok(node_name), Ok(&hitpoint), Ok(cargo_list), Ok(liquid_list), Ok(gas_list)) = (
            entity_entry.get_component::<node::Name>(),
            entity_entry.get_component::<units::Portion<units::Hitpoint>>(),
            entity_entry.get_component::<cargo::StorageList>(),
            entity_entry.get_component::<liquid::StorageList>(),
            entity_entry.get_component::<gas::StorageList>(),
        ) {
            macro_rules! read_storage {
                ($list:expr, $mod:ident, $get_def:ident) => {
                    $list
                        .storages()
                        .iter()
                        .map(|&(id, entity)| {
                            let storage_entry = world
                                .entry_ref(entity)
                                .expect("Storage entity does not exist");
                            let size = storage_entry
                                .get_component::<$mod::StorageSize>()
                                .expect("Storage has no size");
                            let next_size = storage_entry
                                .get_component::<$mod::NextStorageSize>()
                                .expect("Storage has no next size");
                            let lerp_size = $mod::lerp(size, next_size, clock.now());
                            let name = def.$get_def(id).name();
                            (lerp_size, name.clone())
                        })
                        .collect()
                };
            }

            let cargo = read_storage!(cargo_list, cargo, get_cargo);
            let liquid = read_storage!(liquid_list, liquid, get_liquid);
            let gas = read_storage!(gas_list, gas, get_gas);

            Some(Args {
                entity,
                node_name: node_name.name().clone(),
                hitpoint,
                cargo,
                liquid,
                gas,
            })
        } else {
            None
        }
    } else {
        None
    };

    updater_ref.call(Update::SetNodePreview(info));
}

/// Sets up legion ECS for node info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup)
}
