use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{Commands, Query, Res, SystemParam};
use traffloat_physics::util::QueryExt;
use traffloat_proto::proto;

use crate::dock::viewable_info::{show_fluid, show_link};
use crate::dock::{self, plot};
use crate::scene::{FluidTypes, GenericViewable, vehicle};
use crate::util::new_id;

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    vehicle_query:  Query<'w, 's, VehicleData>,
    viewable_query: Query<'w, 's, &'static GenericViewable>,
    commands:       Commands<'w, 's>,
    vehicle_types:  Res<'w, vehicle::Types>,
    fluid_types:    Res<'w, FluidTypes>,
}

#[derive(QueryData)]
struct VehicleData {
    entity:  Entity,
    generic: &'static GenericViewable,
    info:    &'static vehicle::Info,
}

impl UiSystemParam<'_, '_> {
    pub fn ui(&mut self, entity: Entity, ui: &mut egui::Ui, dock: dock::Context) {
        let Some(vehicle_data) = self.vehicle_query.log_get(entity) else {
            ui.label("Object has been unloaded");
            return;
        };

        let ty = self.vehicle_types.types.get(vehicle_data.info.ty);

        ui.heading("Location");
        show_location(ui, &vehicle_data.info.location, &mut self.commands, &self.viewable_query);

        ui.heading("Compartments");
        for (cpmt_index, (cpmt_def, cpmt_info)) in ty
            .iter()
            .flat_map(|ty| &ty.proto.compartments)
            .zip(&vehicle_data.info.compartments)
            .enumerate()
        {
            ui.push_id(new_id!(cpmt_index), |ui| {
                ui.collapsing(&cpmt_def.name, |ui| {
                    show_compartment(
                        ui,
                        &mut self.commands,
                        &self.fluid_types,
                        &vehicle_data,
                        cpmt_index,
                        cpmt_def,
                        cpmt_info,
                    );
                });
            });
        }
    }
}

fn show_location(
    ui: &mut egui::Ui,
    location: &vehicle::Location,
    commands: &mut Commands,
    viewable_query: &Query<&'static GenericViewable>,
) {
    ui.horizontal(|ui| match *location {
        vehicle::Location::Building(building) => {
            show_link(ui, commands, building);
            ui.label("Inside building:");
            if let Some(viewable) = viewable_query.log_get(building) {
                ui.label(&viewable.name);
            }
        }
        vehicle::Location::Rail(rail) => {
            show_link(ui, commands, rail);
            ui.label("On rail:");
            if let Some(viewable) = viewable_query.log_get(rail) {
                ui.label(&viewable.name);
            }
        }
    });
}

fn show_compartment(
    ui: &mut egui::Ui,
    commands: &mut Commands,
    fluid_types: &FluidTypes,
    vehicle_data: &VehicleDataItem,
    cpmt_index: usize,
    cpmt_def: &proto::VehicleTypeCompartment,
    cpmt_info: &vehicle::CompartmentInfo,
) {
    if let Some(fluid) = &cpmt_info.fluid {
        show_fluid(
            ui,
            commands,
            fluid,
            fluid_types,
            |label| format!("Vehicle {} {} {label}", vehicle_data.generic.name, cpmt_def.name),
            |metric| plot::Target::VehicleCompartmentFluid {
                vehicle: vehicle_data.entity,
                compartment: cpmt_index,
                metric,
            },
        );
    }
}
