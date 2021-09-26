//! Renders edge info preview.

use arcstr::ArcStr;
use legion::world::SubWorld;
use legion::{Entity, EntityStore};
use yew::prelude::*;

use super::{duct_editor, Update, UpdaterRef};
use crate::app::icon;
use crate::input;
use crate::render::texture;
use traffloat::def;
use traffloat::units;
use traffloat::{cargo, gas, liquid};
use traffloat::{edge, node};

/// Displays basic info about an edge at a corner of the screen.
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
            Msg::EditDucts => {
                self.props
                    .edit_duct
                    .emit(Some(duct_editor::Args { entity: self.props.args.entity }));
                true
            }
        }
    }

    fn change(&mut self, props: Props) -> ShouldRender {
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        codegen::wasm_dbg!(&self.props.args.liquids);

        fn flow_display(
            size: impl units::RoundedUnit + Into<Html>,
            name: &ArcStr,
            icon: &Option<texture::Icon>,
        ) -> Html {
            html! {
                <>
                    { size }
                    { "/s " }
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
                </>
            }
        }

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
                <p style=style!("margin": "0.5em 0")>{ "Corridor" }</p>
                <button onclick=self.link.callback(|_| Msg::EditDucts)>{ "Edit" }</button>
                <p style=style!("marign": "0.5em 0")>
                    { for self.props.args.liquids.iter().map(|flow| html! {
                        <>
                            { flow_display(flow.flow, &flow.name, &flow.icon) }
                            { " " }
                            { match flow.dir {
                                edge::Direction::FromTo => "forward",
                                edge::Direction::ToFrom => "backward",
                            }}
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
    /// Open the duct editor.
    EditDucts,
}

/// Yew properties for [`Comp`].
#[derive(Clone, Properties)]
pub struct Props {
    /// The yew-independent properties.
    pub args: Args,
    /// Callback to start duct editor
    pub edit_duct: Callback<Option<duct_editor::Args>>,
}

/// Yew-independent properties.
#[derive(Clone)]
pub struct Args {
    /// Entity ID of the edge.
    pub entity: Entity,
    /// Liquid flows along the edge.
    pub liquids: Vec<LiquidFlow>,
}

/// A liquid flow to be rendered.
#[derive(Debug, Clone)]
pub struct LiquidFlow {
    /// Name of the liquid type
    pub name: ArcStr,
    /// Icon name of the liquid type
    pub icon: Option<texture::Icon>,
    /// Flow rate in the pipe.
    pub flow: units::LiquidVolume,
    /// Direction of flow.
    pub dir: edge::Direction,
}

#[codegen::system(Visualize)]
#[read_component(node::Name)]
#[read_component(edge::Id)]
#[read_component(edge::Design)]
#[read_component(liquid::PipeFlow)]
#[thread_local]
fn draw(
    #[resource] hover_target: &input::mouse::HoverTarget,
    #[resource] focus_target: &input::FocusTarget,
    world: &mut SubWorld,
    #[resource] updater_ref: &UpdaterRef,
    #[resource] texture_pool: &Option<texture::Pool>,
    #[resource(no_init)] def: &def::GameDefinition,
) {
    let info = if let Some(entity) = focus_target.entity().or_else(|| hover_target.entity()) {
        let entity_entry = world.entry_ref(entity).expect("Target entity does not exist"); // TODO what if user is hovering over node while deleting it?
        if entity_entry.get_component::<edge::Id>().is_ok() {
            let ducts = entity_entry
                .get_component::<edge::Design>()
                .expect("Edge entity does not have edge::Design component");
            let mut liquids = Vec::new();
            for duct in ducts.ducts() {
                #[allow(clippy::single_match)]
                match duct.ty() {
                    edge::DuctType::Liquid { dir, .. } => {
                        let duct_entry =
                            world.entry_ref(duct.entity()).expect("Duct entity does not exist");
                        let flow = duct_entry
                            .get_component::<liquid::PipeFlow>()
                            .expect("Liquid duct does not have pipe flow component");
                        if let Some(dir) = dir {
                            if let Some(ty) = flow.ty() {
                                let item = def.liquid().get(ty).expect("undefined reference");
                                let name = item.name();
                                let icon = texture_pool.as_ref().and_then(|pool| {
                                    pool.icon(item.texture_src(), item.texture_name())
                                });

                                liquids.push(LiquidFlow {
                                    name: name.clone(),
                                    icon,
                                    flow: flow.value(),
                                    dir,
                                });
                            }
                        }
                    }
                    _ => (), // TODO handle other types
                }
            }
            Some(Args { entity, liquids })
        } else {
            None
        }
    } else {
        None
    };

    updater_ref.call(Update::SetEdgePreview(info));
}

/// Sets up legion ECS for edge info rendering.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup.uses(draw_setup)
}
