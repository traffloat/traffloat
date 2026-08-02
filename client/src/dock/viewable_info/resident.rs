use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{Commands, Query, Res, SystemParam};
use traffloat_physics::util::QueryExt;

use crate::dock::viewable_info::{show_graph_button, show_link};
use crate::dock::{self, plot};
use crate::scene::{GenericViewable, resident, vehicle};
use crate::util::new_id;

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    resident_query:  Query<'w, 's, ResidentData>,
    location_params: ShowLocationParams<'w, 's>,
    commands:        Commands<'w, 's>,
    types:           Res<'w, resident::Types>,
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
        show_location(ui, &resident_data.info.location, &mut self.commands, &self.location_params);

        ui.heading("Attributes");
        show_attributes(
            ui,
            &mut self.commands,
            entity,
            &resident_data.generic.name,
            &self.types,
            &resident_data.info.attributes,
        );
    }
}

#[derive(SystemParam)]
struct ShowLocationParams<'w, 's> {
    viewable_query: Query<'w, 's, &'static GenericViewable>,
    vehicle_query:  Query<'w, 's, &'static vehicle::Info>,
    vehicle_types:  Res<'w, vehicle::Types>,
}

fn show_location(
    ui: &mut egui::Ui,
    location: &resident::Location,
    commands: &mut Commands,
    params: &ShowLocationParams,
) {
    ui.horizontal(|ui| match *location {
        resident::Location::Building(building) => {
            show_link(ui, commands, building);
            ui.label("Inside building:");
            if let Some(viewable) = params.viewable_query.log_get(building) {
                ui.label(&viewable.name);
            }
        }
        resident::Location::Corridor(corridor) => {
            show_link(ui, commands, corridor);
            ui.label("Inside corridor:");
            if let Some(viewable) = params.viewable_query.log_get(corridor) {
                ui.label(&viewable.name);
            }
        }
        resident::Location::Facility { facility, ref slot_name } => {
            show_link(ui, commands, facility);
            ui.label(format!("{slot_name} in facility:"));
            if let Some(viewable) = params.viewable_query.log_get(facility) {
                ui.label(&viewable.name);
            }
        }
        resident::Location::Vehicle { entity, compartment, operator_slot } => {
            show_link(ui, commands, entity);
            let vehicle_type = params
                .vehicle_query
                .log_get(entity)
                .and_then(|info| params.vehicle_types.types.get(info.ty));
            let operator_type = operator_slot
                .zip(vehicle_type)
                .and_then(|(slot, ty)| ty.proto.operator_slots.get(slot))
                .map_or("passenger", |cpmt| cpmt.name.as_str());
            ui.label(format!("As {operator_type} in vehicle"));
            if let Some(viewable) = params.viewable_query.log_get(entity) {
                ui.label(&viewable.name);
            }
            if let Some(ty) = vehicle_type
                && let Some(cpmt) = ty.proto.compartments.get(compartment)
            {
                ui.label(&cpmt.name);
            }
        }
    });
}

fn show_attributes(
    ui: &mut egui::Ui,
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
            ui.push_id(new_id!(ty), |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{}: {value}", def.name));
                    show_graph_button(
                        ui,
                        commands,
                        || format!("Resident {resident_name} {}", def.name),
                        plot::Target::ResidentAttr { resident: entity, ty },
                    );
                });
            });
        }
    }
}
