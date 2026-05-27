use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::system::{Commands, ParamSet, Query, Res, SystemParam};
use egui_material_icons::icons;
use traffloat_physics::util::{Alpha, Beta, GetAb, QueryExt, Which};
use traffloat_proto::proto;

use crate::dock::viewable_info::corridor::display_gate;
use crate::dock::{self, viewable_info};
use crate::scene::{FluidTypes, GenericViewable, building, corridor};
use crate::util::new_id;

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    commands:       Commands<'w, 's>,
    building_query: Query<'w, 's, BuildingData>,
    corridor_query: Query<'w, 's, CorridorData>,
    fluid_types:    Res<'w, FluidTypes>,
}

#[derive(QueryData)]
struct BuildingData {
    generic:         &'static GenericViewable,
    info:            &'static building::Info,
    corridors_alpha: Option<&'static corridor::IsEndpointOf<Alpha>>,
    corridors_beta:  Option<&'static corridor::IsEndpointOf<Beta>>,
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

impl UiSystemParam<'_, '_> {
    pub fn ui(&mut self, entity: Entity, ui: &mut egui::Ui, dock: dock::Context) {
        let Ok(data) = self.building_query.get(entity) else {
            ui.label("Object has been unloaded");
            return;
        };

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
                commands.queue(viewable_info::OpenCommand(peer_building));
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
                commands.queue(viewable_info::OpenCommand(corridor));
            }
            ui.label(&corridor_data.generic.name);
        });

        display_gate(ui, near_detail.open, "Proximal gate");
        display_gate(ui, near_detail.open, "Distal gate");
    });
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
