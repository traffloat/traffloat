use bevy::ecs::entity::Entity;
use bevy::ecs::system::{ParamSet, Query, Res, SystemParam};
use egui_material_icons::icons;
use traffloat_proto::proto;

use crate::dock;
use crate::scene::{FluidTypes, GenericViewable, building};
use crate::util::new_id;

#[derive(SystemParam)]
pub struct UiSystemParam<'w, 's> {
    building_query: Query<'w, 's, &'static building::Info>,
    fluid_types:    Res<'w, FluidTypes>,
}

impl UiSystemParam<'_, '_> {
    pub fn ui(&mut self, entity: Entity, ui: &mut egui::Ui, dock: dock::Context) {
        let Ok(info) = self.building_query.get(entity) else {
            ui.label("Object has been unloaded");
            return;
        };

        if let Some(ambient_fluid) = &info.ambient_fluid {
            egui::CollapsingHeader::new("Ambient fluid").id_salt(new_id!()).show(ui, |ui| {
                show_fluid(ui, ambient_fluid, &self.fluid_types);
            });
        }
    }
}

fn show_fluid(ui: &mut egui::Ui, ambient_fluid: &proto::FluidStorageFull, types: &FluidTypes) {
    ui.label(format!("Volume: {:.2}", ambient_fluid.volume));
    ui.label(format!("Pressure: {:.2}", ambient_fluid.pressure));
    ui.label(format!("Temperature: {:.2} K", ambient_fluid.temperature));

    egui::CollapsingHeader::new("Composition").id_salt(new_id!()).show(ui, |ui| {
        for (id, fraction) in ambient_fluid.types.iter().enumerate() {
            ui.label(format!(
                "{}: {:.2} mol",
                types.0.get(id).map_or("???", |ty| &ty.name),
                fraction * 100.0
            ));
        }
    });
}
