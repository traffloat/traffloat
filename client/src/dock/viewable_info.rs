use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Command, Commands, ParamSet, Query, SystemParam};
use bevy::ecs::world::World;
use egui_material_icons::icons;
use traffloat_proto::proto;

use crate::dock::{self, TabPlacement, viewable_info};
use crate::scene::{FluidTypes, GenericViewable, ViewableKind};
use crate::util::new_id;

mod building;
mod conduit;
mod corridor;
mod facility;

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
        }
    }

    type UiSystemParam<'w, 's> = UiSystemParam<'w, 's>;
    fn ui(
        &mut self,
        mut param: Self::UiSystemParam<'_, '_>,
        ui: &mut egui::Ui,
        dock: dock::Context,
    ) {
        let Ok(generic) = param.generic.get(self.entity) else {
            ui.label("Object has been unloaded");
            return;
        };

        ui.horizontal(|ui| {
            if ui.button(icons::ICON_RECENTER).clicked() {
                param
                    .ps
                    .p0()
                    .queue(dock::camera::FocusCommand { target: self.entity, which: None });
            }
            ui.heading(match generic.kind {
                ViewableKind::Building => "Building:",
                ViewableKind::Corridor => "Corridor:",
                ViewableKind::Facility => "Facility:",
                ViewableKind::Conduit => "Conduit:",
            });
            ui.heading(&generic.name);
            if ui.button(icons::ICON_EDIT).clicked() {
                // TODO edit text
            }
        });

        match generic.kind {
            ViewableKind::Building => {
                let mut building_param = param.ps.p1();
                building_param.ui(self.entity, ui, dock);
            }
            ViewableKind::Corridor => {
                let mut corridor_param = param.ps.p2();
                corridor_param.ui(self.entity, ui, dock);
            }
            ViewableKind::Facility => {
                let mut facility_param = param.ps.p3();
                facility_param.ui(self.entity, ui, dock);
            }
            ViewableKind::Conduit => {
                let mut conduit_param = param.ps.p4();
                conduit_param.ui(self.entity, ui, dock);
            }
        }
    }

    type OnCloseSystemParam<'w, 's> = ();
    type BeforeRenderSystemParam<'w, 's> = ();
}

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    generic: Query<'w, 's, &'static GenericViewable>,
    ps: ParamSet<
        'w,
        's,
        (
            Commands<'w, 's>,
            building::UiSystemParam<'w, 's>,
            corridor::UiSystemParam<'w, 's>,
            facility::UiSystemParam<'w, 's>,
            conduit::UiSystemParam<'w, 's>,
        ),
    >,
}

fn focus_camera_around(entity: Entity) {}

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
