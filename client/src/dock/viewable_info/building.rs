use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::system::{Commands, ParamSet, Query, Res, SystemParam};
use egui_material_icons::icons;
use traffloat_physics::util::{Alpha, Beta, GetAb, QueryExt, Which};
use traffloat_proto::proto;

use crate::dock::viewable_info::corridor::display_gate;
use crate::dock::viewable_info::show_fluid;
use crate::dock::{self, viewable_info};
use crate::scene::facility::BuildingFacilities;
use crate::scene::{FluidTypes, GenericViewable, building, corridor};
use crate::util::new_id;

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    commands:       Commands<'w, 's>,
    building_query: Query<'w, 's, BuildingData>,
    corridor_query: Query<'w, 's, CorridorData>,
    facility_query: Query<'w, 's, FacilityData>,
    fluid_types:    Res<'w, FluidTypes>,
}

#[derive(QueryData)]
struct BuildingData {
    generic:         &'static GenericViewable,
    info:            &'static building::Info,
    corridors_alpha: Option<&'static corridor::IsEndpointOf<Alpha>>,
    corridors_beta:  Option<&'static corridor::IsEndpointOf<Beta>>,
    facilities:      Option<&'static BuildingFacilities>,
}

#[derive(QueryData)]
struct CorridorData {
    generic:        &'static GenericViewable,
    alpha_building: Option<(
        &'static corridor::EndpointRef<Alpha>,
        &'static corridor::GenericEndpointDetails<Alpha>,
    )>,
    beta_building: Option<(
        &'static corridor::EndpointRef<Beta>,
        &'static corridor::GenericEndpointDetails<Beta>,
    )>,
}

impl<'a> GetAb<Option<(Entity, &'a corridor::EndpointDetails)>> for &'a CorridorDataItem<'_, '_> {
    fn alpha(self) -> Option<(Entity, &'a corridor::EndpointDetails)> {
        self.alpha_building.map(|(endpoint, detail)| (endpoint.0, &detail.0))
    }
    fn beta(self) -> Option<(Entity, &'a corridor::EndpointDetails)> {
        self.beta_building.map(|(endpoint, detail)| (endpoint.0, &detail.0))
    }
}

#[derive(QueryData)]
struct FacilityData {
    generic: &'static GenericViewable,
}

impl UiSystemParam<'_, '_> {
    pub fn ui(&mut self, entity: Entity, ui: &mut egui::Ui, dock: dock::Context) {
        let Ok(data) = self.building_query.get(entity) else {
            ui.label("Object has been unloaded");
            return;
        };

        if let Some(facilities) = data.facilities
            && !facilities.is_empty()
        {
            ui.heading("Facilities");
            for facility in facilities.iter() {
                show_facility(ui, dock.id, &mut self.commands, &self.facility_query, facility);
            }
        }

        if data.corridors_alpha.is_some_and(|c| !c.is_empty())
            || data.corridors_beta.is_some_and(|c| !c.is_empty())
        {
            ui.heading("Connections");
            for corridor in data.corridors_alpha.iter().flat_map(|c| c.iter()) {
                show_connection(
                    ui,
                    dock.id,
                    &mut self.commands,
                    &self.building_query,
                    &self.corridor_query,
                    corridor,
                    Alpha,
                );
            }
            for corridor in data.corridors_beta.iter().flat_map(|c| c.iter()) {
                show_connection(
                    ui,
                    dock.id,
                    &mut self.commands,
                    &self.building_query,
                    &self.corridor_query,
                    corridor,
                    Beta,
                );
            }
        }

        if let Some(ambient_fluid) = &data.info.ambient_fluid {
            egui::CollapsingHeader::new("Ambient fluid").id_salt(new_id!(dock.id)).show(ui, |ui| {
                show_fluid(ui, dock.id, ambient_fluid, &self.fluid_types);
            });
        }
    }
}

fn show_connection<Ab: Which>(
    ui: &mut egui::Ui,
    id: egui::Id,
    commands: &mut Commands,
    building_query: &Query<BuildingData>,
    corridor_query: &Query<CorridorData>,
    corridor: Entity,
    which: Ab,
) {
    let Ok(corridor_data) = corridor_query.get(corridor) else { return };
    let (_, near_detail) =
        which.get(&corridor_data).expect("IsEndpointOf implies EndpointRef presence");
    let peer = which.other().get(&corridor_data);

    ui.horizontal(|ui| {
        if let Some((peer_building, peer_detail)) = peer {
            if ui.button(icons::ICON_LINK).clicked() {
                commands.queue(viewable_info::OpenCommand::from_click(peer_building, ui.ctx()));
            }

            if let Some(peer_building_data) = building_query.log_get(peer_building) {
                ui.label(&peer_building_data.generic.name);
            }
        }
    });

    ui.indent(new_id!(id), |ui| {
        ui.horizontal(|ui| {
            ui.label("through");
            if ui.small_button(icons::ICON_LINK).clicked() {
                commands.queue(viewable_info::OpenCommand::from_click(corridor, ui.ctx()));
            }
            ui.label(&corridor_data.generic.name);
        });

        display_gate(ui, near_detail.open, "Proximal gate");
        display_gate(ui, near_detail.open, "Distal gate");
    });
}

fn show_facility(
    ui: &mut egui::Ui,
    id: egui::Id,
    commands: &mut Commands,
    facility_query: &Query<FacilityData>,
    facility_entity: Entity,
) {
    let Some(facility_data) = facility_query.log_get(facility_entity) else { return };

    ui.horizontal(|ui| {
        if ui.button(icons::ICON_LINK).clicked() {
            commands.queue(viewable_info::OpenCommand::from_click(facility_entity, ui.ctx()));
        }

        ui.label(&facility_data.generic.name);
    });
}
