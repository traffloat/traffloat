use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::relationship::RelationshipTarget;
use bevy::ecs::system::{Commands, Query, Res, SystemParam};
use egui_material_icons::icons;
use traffloat_physics::util::{Alpha, Beta, QueryExt, Which};

use crate::dock::viewable_info::{show_fluid, show_link};
use crate::dock::{self, viewable_info};
use crate::scene::conduit::CorridorConduits;
use crate::scene::{FluidTypes, GenericViewable, corridor};
use crate::util::new_id;

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    corridor_query: Query<'w, 's, CorridorData>,
    building_query: Query<'w, 's, &'static GenericViewable>,
    conduit_query:  Query<'w, 's, &'static GenericViewable>,
    fluid_types:    Res<'w, FluidTypes>,
    commands:       Commands<'w, 's>,
}

#[derive(QueryData)]
struct CorridorData {
    info:     &'static corridor::Info,
    alpha:    Option<EndpointData<Alpha>>,
    beta:     Option<EndpointData<Beta>>,
    conduits: Option<&'static CorridorConduits>,
}

#[derive(QueryData)]
struct EndpointData<Ab: Which> {
    building: &'static corridor::EndpointRef<Ab>,
    detail:   &'static corridor::GenericEndpointDetails<Ab>,
}

impl UiSystemParam<'_, '_> {
    pub fn ui(&mut self, entity: Entity, ui: &mut egui::Ui, dock: dock::Context) {
        let Ok(data) = self.corridor_query.get(entity) else {
            ui.label("Object has been unloaded");
            return;
        };

        ui.heading("Conduits");
        for conduit in data.conduits.into_iter().flat_map(RelationshipTarget::iter) {
            show_conduit(ui, dock.id, &self.conduit_query, &mut self.commands, conduit);
        }

        ui.heading("Connections");
        if let Some(alpha) = data.alpha {
            show_connection(ui, dock.id, &self.building_query, &mut self.commands, alpha);
        }
        if let Some(beta) = data.beta {
            show_connection(ui, dock.id, &self.building_query, &mut self.commands, beta);
        }

        if let Some(ambient_fluid) = &data.info.ambient_fluid {
            egui::CollapsingHeader::new("Ambient fluid").id_salt(new_id!(dock.id)).show(ui, |ui| {
                show_fluid(ui, dock.id, ambient_fluid, &self.fluid_types);
            });
        }
    }
}

fn show_conduit(
    ui: &mut egui::Ui,
    id: egui::Id,
    conduit_query: &Query<&GenericViewable>,
    commands: &mut Commands,
    entity: Entity,
) {
    let Some(conduit_viewable) = conduit_query.log_get(entity) else { return };

    ui.horizontal(|ui| {
        show_link(ui, commands, entity);
        ui.label(&conduit_viewable.name);
    });
}

fn show_connection(
    ui: &mut egui::Ui,
    id: egui::Id,
    building_query: &Query<&GenericViewable>,
    commands: &mut Commands,
    data: EndpointDataItem<impl Which>,
) {
    let Some(building_info) = building_query.log_get(data.building.0) else { return };

    ui.horizontal(|ui| {
        show_link(ui, commands, data.building.0);
        ui.label(&building_info.name);
    });

    ui.indent(new_id!(id), |ui| {
        let detail = &data.detail.0;
        display_gate(ui, detail.open, "Gate");
    });
}

pub(super) fn display_gate(ui: &mut egui::Ui, open: bool, text: &str) {
    let mut open_var = open;
    ui.checkbox(&mut open_var, format!("{text} {}", if open { "open" } else { "closed" }));
    // TODO send open/close request on change
}
