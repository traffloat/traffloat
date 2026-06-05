use bevy::ecs::entity::Entity;
use bevy::ecs::query::QueryData;
use bevy::ecs::system::{Commands, ParamSet, Query, Res, SystemParam};
use bevy::ecs::world::World;
use egui_material_icons::icons;
use traffloat_physics::util::{Alpha, Beta, QueryExt, Which};
use traffloat_proto::proto;

use crate::dock::viewable_info::show_fluid;
use crate::dock::{self, viewable_info};
use crate::scene::{FluidTypes, GenericViewable, conduit};
use crate::util::new_id;

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    conduit_query:  Query<'w, 's, ConduitData>,
    corridor_query: Query<'w, 's, &'static GenericViewable>,
    fluid_types:    Res<'w, FluidTypes>,
    commands:       Commands<'w, 's>,
}

#[derive(QueryData)]
struct ConduitData {
    info:     &'static conduit::Info,
    corridor: &'static conduit::ConduitCorridor,
}

impl UiSystemParam<'_, '_> {
    pub fn ui(&mut self, entity: Entity, ui: &mut egui::Ui, dock: dock::Context) {
        let Ok(data) = self.conduit_query.get(entity) else {
            ui.label("Object has been unloaded");
            return;
        };

        ui.heading("Located in");
        show_corridor(ui, dock.id, &self.corridor_query, &mut self.commands, data.corridor.0);

        // TODO facility connections

        if let Some(ambient_fluid) = &data.info.stored_fluid {
            egui::CollapsingHeader::new("Stored fluid").id_salt(new_id!(dock.id)).show(ui, |ui| {
                show_fluid(ui, dock.id, ambient_fluid, &self.fluid_types);
            });
        }
    }
}

fn show_corridor(
    ui: &mut egui::Ui,
    id: egui::Id,
    corridor_query: &Query<&GenericViewable>,
    commands: &mut Commands,
    corridor_entity: Entity,
) {
    let Some(corridor_info) = corridor_query.log_get(corridor_entity) else { return };

    ui.horizontal(|ui| {
        if ui.button(icons::ICON_LINK).clicked() {
            commands.queue(viewable_info::OpenCommand::from_click(corridor_entity, ui.ctx()));
        }
        ui.label(&corridor_info.name);
    });
}
