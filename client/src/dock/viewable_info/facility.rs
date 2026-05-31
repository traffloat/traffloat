use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{Commands, ParamSet, Query, Res, SystemParam};
use bevy::ecs::world::World;
use egui_material_icons::icons;
use traffloat_physics::util::{Alpha, Beta, QueryExt, Which};
use traffloat_proto::proto;

use crate::dock::viewable_info::show_fluid;
use crate::dock::{self, viewable_info};
use crate::scene::{FluidTypes, GenericViewable, facility};
use crate::util::new_id;

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    facility_query: Query<'w, 's, FacilityData>,
    building_query: Query<'w, 's, &'static GenericViewable>,
    fluid_types:    Res<'w, FluidTypes>,
    commands:       Commands<'w, 's>,
}

#[derive(QueryData)]
struct FacilityData {
    info:     &'static facility::Info,
    building: &'static facility::FacilityBuilding,
}

impl UiSystemParam<'_, '_> {
    pub fn ui(&mut self, entity: Entity, ui: &mut egui::Ui, dock: dock::Context) {
        let Ok(data) = self.facility_query.get(entity) else {
            ui.label("Object has been unloaded");
            return;
        };

        ui.heading("Connections");
        show_building(ui, dock.id, &self.building_query, &mut self.commands, data.building.0);
        // TODO conduit connections
        // TODO intra-building connections

        if let Some(ambient_fluid) = &data.info.stored_fluid {
            egui::CollapsingHeader::new("Stored fluid").id_salt(new_id!(dock.id)).show(ui, |ui| {
                show_fluid(ui, dock.id, ambient_fluid, &self.fluid_types);
            });
        }
    }
}

fn show_building(
    ui: &mut egui::Ui,
    id: egui::Id,
    building_query: &Query<&GenericViewable>,
    commands: &mut Commands,
    building_entity: Entity,
) {
    let Some(building_info) = building_query.log_get(building_entity) else { return };

    ui.horizontal(|ui| {
        if ui.button(icons::ICON_LINK).clicked() {
            commands.queue(viewable_info::OpenCommand::from_click(building_entity, ui.ctx()));
        }
        ui.label(&building_info.name);
    });
}
