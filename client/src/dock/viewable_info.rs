use bevy::app::{App, Plugin};
use bevy::ecs::entity::Entity;
use bevy::ecs::message::MessageWriter;
use bevy::ecs::system::{Command, Commands, ParamSet, Query, SystemParam};
use bevy::ecs::world::World;
use egui_material_icons::icons;
use traffloat_proto::proto;
use traffloat_scene::{self, FluidTypes, GenericViewable, OutboundRequest, ProtoId, ViewableKind};
use traffloat_util::QueryExt;

use crate::dock::{self, DockCommand, TabPlacement, plot, viewable_info};
use crate::util::new_id;

mod building;
mod conduit;
mod corridor;
mod facility;
mod resident;
mod vehicle;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, _app: &mut App) {}
}

pub struct Tab {
    pub entity: Entity,
    ui_state:   UiState,
}

impl dock::Tab for Tab {
    type TitleSystemParam<'w, 's> = Query<'w, 's, &'static GenericViewable>;
    fn title(&self, param: Self::TitleSystemParam<'_, '_>) -> String {
        let Ok(viewable) = param.get(self.entity) else {
            return "[Invalid entity]".to_string();
        };
        match viewable.kind {
            ViewableKind::Building => format!("Building: {}", viewable.name),
            ViewableKind::Corridor => format!("Corridor: {}", viewable.name),
            ViewableKind::Facility => format!("Facility: {}", viewable.name),
            ViewableKind::Conduit => format!("Conduit: {}", viewable.name),
            ViewableKind::Resident => format!("Resident: {}", viewable.name),
            ViewableKind::Vehicle => format!("Vehicle: {}", viewable.name),
        }
    }

    type UiSystemParam<'w, 's> = UiSystemParam<'w, 's>;
    fn ui(
        &mut self,
        mut param: Self::UiSystemParam<'_, '_>,
        ui: &mut egui::Ui,
        dock: dock::Context,
    ) {
        let mut generic = param.ps.p0();

        let Ok((viewable, &ProtoId(id))) = generic.viewable_query.get(self.entity) else {
            ui.label("Object has been unloaded");
            return;
        };

        ui.horizontal(|ui| {
            if ui.button(icons::ICON_RECENTER).on_hover_text("Focus").clicked() {
                generic
                    .commands
                    .queue(dock::camera::FocusCommand { target: self.entity, which: None });
            }
            ui.heading(match viewable.kind {
                ViewableKind::Building => "Building:",
                ViewableKind::Corridor => "Corridor:",
                ViewableKind::Facility => "Facility:",
                ViewableKind::Conduit => match generic.conduit_query.log_get(self.entity) {
                    Some(info) => match info.ty {
                        proto::ConduitType::FluidPipe => "Pipe:",
                        proto::ConduitType::VehicleRail => "Rail:",
                    },
                    None => "Invalid conduit:",
                },
                ViewableKind::Resident => "Resident:",
                ViewableKind::Vehicle => "Vehicle:",
            });
            match self.ui_state.name_edit {
                None => {
                    ui.heading(&viewable.name);
                    if ui.button(icons::ICON_EDIT).on_hover_text("Rename").clicked() {
                        self.ui_state.name_edit = Some(viewable.name.clone());
                    }
                }
                Some(ref mut name) => {
                    let resp = egui::TextEdit::singleline(name)
                        .font(egui::TextStyle::Heading)
                        .show(ui)
                        .response;
                    let submit =
                        resp.lost_focus() && resp.ctx.input(|i| i.key_pressed(egui::Key::Enter));
                    let mut cancel = resp.lost_focus();

                    let is_valid = !name.is_empty() && name != &viewable.name;
                    ui.add_enabled_ui(is_valid, |ui| {
                        if ui.button(icons::ICON_CHECK).on_hover_text("Confirm rename").clicked()
                            || submit
                        {
                            generic.request_writer.write(OutboundRequest {
                                body: proto::RenameViewable { id, name: name.clone() }.into(),
                            });
                            cancel = true;
                        }
                    });

                    if cancel {
                        self.ui_state.name_edit = None;
                    }
                }
            }
        });

        let kind = viewable.kind;
        match kind {
            ViewableKind::Building => {
                let mut param = param.ps.p1();
                param.ui(self.entity, ui, dock);
            }
            ViewableKind::Corridor => {
                let mut param = param.ps.p2();
                param.ui(self.entity, ui, dock);
            }
            ViewableKind::Facility => {
                let mut param = param.ps.p3();
                param.ui(self.entity, ui, dock);
            }
            ViewableKind::Conduit => {
                let mut param = param.ps.p4();
                param.ui(self.entity, ui, dock);
            }
            ViewableKind::Resident => {
                let mut param = param.ps.p5();
                param.ui(self.entity, ui, dock);
            }
            ViewableKind::Vehicle => {
                let mut param = param.ps.p6();
                param.ui(self.entity, ui, dock);
            }
        }
    }

    type OnCloseSystemParam<'w, 's> = ();
    type BeforeRenderSystemParam<'w, 's> = ();
}

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    ps: ParamSet<
        'w,
        's,
        (
            GenericUiSystemParams<'w, 's>,
            building::UiSystemParam<'w, 's>,
            corridor::UiSystemParam<'w, 's>,
            facility::UiSystemParam<'w, 's>,
            conduit::UiSystemParam<'w, 's>,
            resident::UiSystemParam<'w, 's>,
            vehicle::UiSystemParam<'w, 's>,
        ),
    >,
}

#[derive(SystemParam)]
struct GenericUiSystemParams<'w, 's> {
    commands:       Commands<'w, 's>,
    conduit_query:  Query<'w, 's, &'static traffloat_scene::conduit::Info>,
    viewable_query: Query<'w, 's, (&'static GenericViewable, &'static ProtoId)>,
    request_writer: MessageWriter<'w, OutboundRequest>,
}

#[derive(Default)]
struct UiState {
    name_edit: Option<String>,
}

pub struct OpenCommand {
    pub entity:    Entity,
    pub force_new: bool,
}

impl OpenCommand {
    pub fn from_click(entity: Entity, ctx: &egui::Context) -> Self {
        let force_new = ctx.input(|input| input.modifiers.command);
        Self { entity, force_new }
    }
}

impl Command for OpenCommand {
    type Out = ();
    fn apply(self, world: &mut World) {
        world.resource_mut::<dock::State>().focus_or_create(
            || viewable_info::Tab { entity: self.entity, ui_state: UiState::default() }.into(),
            dock::ReplaceTab(|state| state.tab.is_viewable_info())
                .only_if(!self.force_new)
                .or(dock::Split { split: egui_dock::Split::Right, ratio: 0.7 }
                    .at(|state| state.tab.is_camera()))
                .or_always(dock::Split { split: egui_dock::Split::Right, ratio: 0.7 }),
        );
    }
}

fn show_fluid(
    ui: &mut egui::Ui,
    commands: &mut Commands,
    fluid: &proto::FluidStorageDetail,
    types: &FluidTypes,
    make_name: impl Fn(&str) -> String,
    make_plot_target: impl Fn(plot::FluidMetric) -> plot::Target,
) {
    ui.label(format!("Volume: {:.2}", fluid.volume));
    if let Some(pressure) = fluid.pressure {
        ui.horizontal(|ui| {
            ui.label(format!("Pressure: {pressure:.2}"));
            ui.push_id(new_id!(), |ui| {
                show_graph_button(
                    ui,
                    commands,
                    || make_name("Pressure"),
                    make_plot_target(plot::FluidMetric::Pressure),
                );
            });
        });
    }
    if let Some(temperature) = fluid.temperature {
        ui.horizontal(|ui| {
            ui.label(format!("Temperature: {temperature:.2} K"));
            ui.push_id(new_id!(), |ui| {
                show_graph_button(
                    ui,
                    commands,
                    || make_name("Temperature"),
                    make_plot_target(plot::FluidMetric::Temperature),
                );
            });
        });
    }

    if let Some(data) = &fluid.types {
        egui::CollapsingHeader::new("Composition").id_salt(new_id!()).show(ui, |ui| {
            for (ty, &moles) in data.iter().enumerate() {
                ui.push_id(new_id!(ty), |ui| {
                    ui.horizontal(|ui| {
                        let ty_name = types.0.get(ty).map_or("???", |ty| &ty.name);
                        ui.label(format!(
                            "{ty_name}: {moles:.2} mol ({} mol/m\u{b3})",
                            moles / fluid.volume,
                        ));
                        show_graph_button(
                            ui,
                            commands,
                            || make_name(&format!("{ty_name} concentration")),
                            make_plot_target(plot::FluidMetric::MolarConc { ty }),
                        );
                    });
                });
            }
        });
    }
}

fn show_graph_button(
    ui: &mut egui::Ui,
    commands: &mut Commands,
    make_name: impl Fn() -> String,
    target: plot::Target,
) {
    let resp = ui.button(icons::ICON_CHART_DATA).on_hover_text("Click to open chart for {label}");
    if resp.clicked() {
        let name = make_name();
        commands.queue(DockCommand(move |dock| {
            let tab = plot::Tab::new(name, target.clone());
            dock.focus_or_create(
                || dock::TabEnum::Plot(tab),
                dock::ReplaceTab(move |tab| match &tab.tab {
                    dock::TabEnum::Plot(tab) => tab.targets.contains(&target),
                    _ => false,
                })
                .or_always(dock::NewWindow),
            );
        }));
    }
}

fn show_link(ui: &mut egui::Ui, commands: &mut Commands, entity: Entity) {
    if ui.button(icons::ICON_LINK).on_hover_text("View").clicked() {
        commands.queue(viewable_info::OpenCommand::from_click(entity, ui.ctx()));
    }
}

fn show_link_small(ui: &mut egui::Ui, commands: &mut Commands, entity: Entity) {
    if ui.small_button(icons::ICON_LINK).on_hover_text("View").clicked() {
        commands.queue(viewable_info::OpenCommand::from_click(entity, ui.ctx()));
    }
}
