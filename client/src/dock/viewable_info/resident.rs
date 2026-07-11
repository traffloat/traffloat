use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{Commands, Query, Res, SystemParam};
use egui_material_icons::icons;
use traffloat_physics::util::QueryExt;

use crate::dock::viewable_info::show_graph_button;
use crate::dock::{self, plot, viewable_info};
use crate::scene::{GenericViewable, resident};
use crate::util::new_id;

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    resident_query: Query<'w, 's, ResidentData>,
    viewable_query: Query<'w, 's, &'static GenericViewable>,
    commands:       Commands<'w, 's>,
    types:          Res<'w, resident::Types>,
}

#[derive(QueryData)]
struct ResidentData {
    generic: &'static GenericViewable,
    info:    &'static resident::Info,
}

impl UiSystemParam<'_, '_> {
    pub fn ui(&mut self, entity: Entity, ui: &mut egui::Ui, dock: dock::Context) {
        let Some(resident_data) = self.resident_query.log_get(entity) else {
            ui.label("Object has been unloaded");
            return;
        };

        ui.heading("Location");
        show_location(
            ui,
            dock.id,
            &resident_data.info.location,
            &mut self.commands,
            &self.viewable_query,
        );

        ui.heading("Attributes");
        show_attributes(
            ui,
            dock.id,
            &mut self.commands,
            entity,
            &resident_data.generic.name,
            &self.types,
            &resident_data.info.attributes,
        );
    }
}

fn show_location(
    ui: &mut egui::Ui,
    id: egui::Id,
    location: &resident::Location,
    commands: &mut Commands,
    viewable_query: &Query<&'static GenericViewable>,
) {
    ui.horizontal(|ui| match *location {
        resident::Location::Building(building) => {
            if ui.button(icons::ICON_LINK).on_hover_text("View").clicked() {
                commands.queue(viewable_info::OpenCommand::from_click(building, ui.ctx()));
            }
            ui.label("Inside building:");
            if let Some(viewable) = viewable_query.log_get(building) {
                ui.label(&viewable.name);
            }
        }
        resident::Location::Corridor(corridor) => {
            if ui.button(icons::ICON_LINK).on_hover_text("View").clicked() {
                commands.queue(viewable_info::OpenCommand::from_click(corridor, ui.ctx()));
            }
            ui.label("Inside corridor:");
            if let Some(viewable) = viewable_query.log_get(corridor) {
                ui.label(&viewable.name);
            }
        }
        resident::Location::Facility { facility, ref slot_name } => {
            if ui.button(icons::ICON_LINK).on_hover_text("View").clicked() {
                commands.queue(viewable_info::OpenCommand::from_click(facility, ui.ctx()));
            }
            ui.label(format!("{slot_name} in facility:"));
            if let Some(viewable) = viewable_query.log_get(facility) {
                ui.label(&viewable.name);
            }
        }
    });
}

fn show_attributes(
    ui: &mut egui::Ui,
    id: egui::Id,
    commands: &mut Commands,
    entity: Entity,
    resident_name: &str,
    types: &resident::Types,
    attributes: &[Option<f32>],
) {
    for (ty, value) in attributes.iter().enumerate() {
        if let Some(value) = value
            && let Some(def) = types.types.get(ty)
        {
            ui.horizontal(|ui| {
                ui.label(format!("{}: {value}", def.name));
                show_graph_button(
                    ui,
                    new_id!(id, ty),
                    commands,
                    || format!("Resident {resident_name} {}", def.name),
                    plot::Target::ResidentAttr { resident: entity, ty },
                );
            });
        }
    }
}
