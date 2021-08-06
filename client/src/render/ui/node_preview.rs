//! Renders node info preview.

use arcstr::ArcStr;
use legion::world::SubWorld;
use legion::{Entity, EntityStore};
use smallvec::SmallVec;
use yew::prelude::*;

use super::{Update, UpdaterRef};
use crate::app::icon;
use crate::input;
use crate::render::texture;
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
        use crate::render::texture::Icon;
        use traffloat::units::RoundedUnit;

        fn storage_display(
            size: impl RoundedUnit + Into<Html>,
            name: &ArcStr,
            icon: &Option<Icon>,
        ) -> Html {
            html! {
                <>
                    { size.round(2) }
                    { " " }
                    { for icon.as_ref().map(|icon| html! {
                        <icon::Comp
                            atlas_path=icon.url.to_string()
                            atlas_width=icon.dim.0
                            atlas_height=icon.dim.1
                            x0=icon.pos.x()
                            y0=icon.pos.y()
                            x1=icon.pos.x() + icon.pos.width()
                            y1=icon.pos.y() + icon.pos.height()
                            out_width=24
                            out_height=24
                            text=name.to_string()
                            />
                    }) }
                    { for icon.is_none().then(|| name) }
                    <br />
                </>
            }
        }

        html! {
            <div style="
                position: absolute;
                bottom: 0;
                left: 0;
                width: 15em; height: 10em;
                color: black;
                pointer-events: auto;
                background-color: white;
                font-size: large;
                padding: 1em 0.2em;
                overflow-y: auto;
            ">
                <p
                    onclick=self.link.callback(Msg::HelpButton)
                    style="
                        margin: 0.5em 0;
                        cursor: help;
                    "
                >
                    { &self.props.args.node_name }
                </p>
                <p style="margin: 0.5em 0;">
                    { self.props.args.hitpoint }
                </p>
                <p style="margin: 0.5em 0;">
                    { for self.props.args.cargo.iter().map(|(size, name, icon)| storage_display(*size, name, icon)) }
                </p>
                <p style="margin: 0.5em 0;">
                    { for self.props.args.liquid.iter().map(|(size, name, icon)| storage_display(*size, name, icon)) }
                </p>
                <p style="margin: 0.5em 0;">
                    { for self.props.args.gas.iter().map(|(size, name, icon)| storage_display(*size, name, icon)) }
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
    pub cargo: SmallVec<[(units::CargoSize, ArcStr, Option<texture::Icon>); 4]>,
    /// Liquids stored in the targeted node.
    pub liquid: SmallVec<[(units::LiquidVolume, ArcStr, Option<texture::Icon>); 4]>,
    /// Gases stored in the targeted node.
    pub gas: SmallVec<[(units::GasVolume, ArcStr, Option<texture::Icon>); 4]>,
}

#[codegen::system]
#[read_component(node::Name)]
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
    #[resource(no_init)] def: &GameDefinition,
    #[resource] hover_target: &input::mouse::HoverTarget,
    #[resource] focus_target: &input::FocusTarget,
    world: &mut SubWorld,
    #[resource] updater_ref: &UpdaterRef,
    #[resource] clock: &Clock,
    #[resource] texture_pool: &Option<texture::Pool>,
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
                ($list:expr, $mod:ident) => {
                    $list
                        .storages()
                        .iter()
                        .map(|(id, entity)| {
                            let storage_entry = world
                                .entry_ref(*entity)
                                .expect("Storage entity does not exist");
                            let size = storage_entry
                                .get_component::<$mod::StorageSize>()
                                .expect("Storage has no size");
                            let next_size = storage_entry
                                .get_component::<$mod::NextStorageSize>()
                                .expect("Storage has no next size");
                            let lerp_size = $mod::lerp(size, next_size, clock.now());
                            let item = def.$mod().get(id).expect("Undefined reference");
                            let name = item.name();
                            let icon = texture_pool.as_ref().and_then(|pool| {
                                pool.icon(item.texture_src(), item.texture_name())
                            });
                            (lerp_size, name.clone(), icon)
                        })
                        .collect()
                };
            }

            let cargo = read_storage!(cargo_list, cargo);
            let gas = read_storage!(gas_list, gas);

            let liquid = liquid_list
                .storages()
                .iter()
                .map(|entity| {
                    let storage_entry = world
                        .entry_ref(*entity)
                        .expect("Storage entity does not exist");
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
                    let item = def
                        .liquid()
                        .get(storage.liquid())
                        .expect("undefined reference");
                    let name = item.name();
                    let icon = texture_pool
                        .as_ref()
                        .and_then(|pool| pool.icon(item.texture_src(), item.texture_name()));
                    (lerp_size, name.clone(), icon)
                })
                .collect();

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
