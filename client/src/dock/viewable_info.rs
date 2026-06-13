use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Command, Commands, ParamSet, Query, SystemParam};
use bevy::ecs::world::World;
use egui_material_icons::icons;
use traffloat_physics::util::QueryExt;
use traffloat_proto::proto;

use crate::dock::{self, TabPlacement, viewable_info};
use crate::scene::{self, FluidTypes, GenericViewable, ViewableKind};
use crate::util::new_id;

mod building;
mod conduit;
mod corridor;
mod facility;
mod resident;

pub struct Tab {
    pub entity: Entity,
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

        let Ok(viewable) = generic.viewable_query.get(self.entity) else {
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
                        proto::ConduitType::FluidPipe => "Fluid pipe:",
                    },
                    None => "Invalid conduit:",
                },
                ViewableKind::Resident => "Resident:",
            });
            ui.heading(&viewable.name);
            if ui.button(icons::ICON_EDIT).on_hover_text("Rename").clicked() {
                // TODO edit text
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
        ),
    >,
}

#[derive(SystemParam)]
struct GenericUiSystemParams<'w, 's> {
    commands:       Commands<'w, 's>,
    conduit_query:  Query<'w, 's, &'static scene::conduit::Info>,
    viewable_query: Query<'w, 's, &'static GenericViewable>,
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
    fn apply(self, world: &mut World) {
        world.resource_mut::<dock::State>().focus_or_create(
            || viewable_info::Tab { entity: self.entity }.into(),
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
    id: egui::Id,
    ambient_fluid: &proto::FluidStorageFull,
    types: &FluidTypes,
) {
    ui.label(format!("Volume: {:.2}", ambient_fluid.volume));
    ui.label(format!("Pressure: {:.2}", ambient_fluid.pressure));
    ui.label(format!("Temperature: {:.2} K", ambient_fluid.temperature));

    egui::CollapsingHeader::new("Composition").id_salt(new_id!(id)).show(ui, |ui| {
        for (id, fraction) in ambient_fluid.types.iter().enumerate() {
            ui.label(format!(
                "{}: {:.2} mol",
                types.0.get(id).map_or("???", |ty| &ty.name),
                fraction * 100.0
            ));
        }
    });
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
